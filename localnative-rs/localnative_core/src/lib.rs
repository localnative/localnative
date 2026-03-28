/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::runtime::Runtime;

fn global_runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("Failed to create tokio runtime"))
}

pub mod db;
pub mod discovery;
pub mod error;
pub mod export;
pub mod import;
pub mod rpc;

// Re-export error types at crate root for convenience.
pub use error::{DatabaseError, Error, ProcessError, SyncError, ValidationError};

#[cfg(target_os = "android")]
pub mod android {
    use super::*;
    use jni::objects::{JClass, JString};
    use jni::sys::jstring;
    use jni::JNIEnv;

    #[no_mangle]
    pub unsafe extern "C" fn Java_app_localnative_android_RustBridge_localnativeRun(
        env: JNIEnv,
        _: JClass,
        json_input: JString,
    ) -> jstring {
        let json = match env.get_string(json_input) {
            Ok(s) => s.to_string_lossy().into_owned(),
            Err(_) => {
                return env
                    .new_string(r#"{"error": "Invalid json input string"}"#)
                    .map(|s| s.into_raw())
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        let result = run_async(&json);
        match env.new_string(result) {
            Ok(output) => output.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }
}

/// # Safety
///
/// `json_input` must be a valid, non-null pointer to a nul-terminated C string that remains
/// valid for the duration of this call. The returned pointer must be freed with
/// [`localnative_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn localnative_run(json_input: *const c_char) -> *mut c_char { unsafe {
    let c_str = CStr::from_ptr(json_input);
    let json = match c_str.to_str() {
        Ok(s) => run_async(s),
        Err(_) => r#"{"error": "Invalid UTF-8 in input"}"#.to_string(),
    };

    match CString::new(json) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => CString::new(r#"{"error": "Response contained null byte"}"#)
            .unwrap_or_default()
            .into_raw(),
    }
}}

/// # Safety
///
/// `s` must be a pointer previously returned by [`localnative_run`], or null. After this call
/// the pointer is invalid and must not be used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn localnative_free(s: *mut c_char) { unsafe {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum Cmd {
    Server(CmdRpcServer),
    ClientSync(CmdRpcClient),
    ClientStopServer(CmdRpcClient),
    #[serde(untagged)]
    DbCmd(db::models::Cmd),
}

#[test]
fn test_serde() {
    let cmd = Cmd::DbCmd(db::models::Cmd::Insert(db::models::CmdInsert {
        title: "Test Title".into(),
        url: "http://example.com".into(),
        tags: "tag1,tag2".into(),
        description: "This is a test description".into(),
        comments: "Comment 1".into(),
        annotations: "Annotation 1".into(),
        limit: 10,
        offset: 0,
        is_public: true,
    }));
    let json = serde_json::to_string_pretty(&cmd).expect("Failed to serialize command");
    println!("{:#}", json);

    let cmd = Cmd::DbCmd(db::models::Cmd::Search(db::models::CmdSearch {
        query: "hello".into(),
        limit: 10,
        offset: 0,
    }));
    let json = serde_json::to_string_pretty(&cmd).expect("Failed to serialize command");
    println!("{:#}", json);
    let cmd = serde_json::from_str::<'_, Cmd>(&json).unwrap();
    println!("{:#?}", cmd);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdSyncViaAttach {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdRpcClient {
    pub addr: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdRpcServer {
    pub addr: String,
}

/// Parse `text` as a JSON [`Cmd`], execute it, and return the result as a JSON string.
/// Errors are serialized into the returned string rather than propagated.
pub async fn run(text: &str) -> String {
    match serde_json::from_str::<Cmd>(text) {
        Ok(cmd) => match process(cmd).await {
            Ok(rs) => rs,
            Err(err) => serialize_error(err, text),
        },
        Err(e) => serialize_error(ProcessError::Serde(e), text),
    }
}

/// Synchronous wrapper around [`run`] — blocks the current thread on the shared Tokio runtime.
/// Intended for use from C FFI and other non-async callers.
pub fn run_sync(text: &str) -> String {
    global_runtime().block_on(run(text))
}

#[derive(Serialize)]
struct SerializeError<'s> {
    #[serde(flatten)]
    error: ProcessError,
    source_text: &'s str,
}
#[test]
fn test_serialize_error() {
    let err = SerializeError {
        error: ProcessError::Io(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "addres in use.",
        )),
        source_text: "source_text",
    };
    println!("json: {:#}", serde_json::to_string(&err).unwrap())
}

fn serialize_error(err: ProcessError, text: &str) -> String {
    let err = SerializeError {
        error: err,
        source_text: text,
    };
    serde_json::to_string(&err).unwrap_or_else(|_| "Serialization error".to_string())
}

#[derive(Serialize)]
struct ServerResponse {
    server: String,
}

#[derive(Serialize)]
struct ClientSyncResponse {
    #[serde(rename = "client-sync")]
    client_sync: String,
}

#[derive(Serialize)]
struct ClientStopServerResponse {
    #[serde(rename = "client-stop-server")]
    client_stop_server: String,
}

async fn process(cmd: Cmd) -> Result<String, ProcessError> {
    tracing::debug!(?cmd, "processing command");
    let conn = db::init_db()?;

    let result = match cmd {
        Cmd::Server(s) => {
            let pool = Arc::new(Mutex::new(conn));
            crate::rpc::start(&s.addr, &pool).await?;
            Ok(serde_json::to_string(&ServerResponse {
                server: "started".to_string(),
            })?)
        }
        Cmd::ClientSync(s) => {
            let pool = Arc::new(Mutex::new(conn));
            let resp = crate::rpc::sync(&s.addr, &pool).await?;
            Ok(serde_json::to_string(&ClientSyncResponse {
                client_sync: resp,
            })?)
        }
        Cmd::ClientStopServer(s) => {
            let pool = Arc::new(Mutex::new(conn));
            let resp = crate::rpc::stop_server(&s.addr, &pool).await?;
            Ok(serde_json::to_string(&ClientStopServerResponse {
                client_stop_server: resp,
            })?)
        }
        Cmd::DbCmd(db_cmd) => Ok(db::process_cmd(db_cmd, &conn)?),
    };

    result
}

fn run_async(text: &str) -> String {
    global_runtime().block_on(run(text))
}
