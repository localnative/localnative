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
use thiserror::Error;
use tokio::runtime::Runtime;

pub mod db;
mod error;
pub mod rpc;

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
        let json = env
            .get_string(json_input)
            .expect("Invalid json input string!")
            .to_string_lossy()
            .into_owned();

        let result = run_async(&json);
        let output = env
            .new_string(result)
            .expect("Couldn't create java output string!");

        output.into_inner()
    }
}

#[no_mangle]
pub unsafe extern "C" fn localnative_run(json_input: *const c_char) -> *mut c_char {
    let c_str = CStr::from_ptr(json_input);
    let json = match c_str.to_str() {
        Ok(s) => run_async(s),
        Err(_) => r#"{"error": "Invalid UTF-8 in input"}"#.to_string(),
    };

    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn localnative_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

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

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("database error: {0}")]
    DbError(#[from] db::DbError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("address parse error: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error("rpc error: {0}")]
    RpcError(#[from] tarpc::client::RpcError),
    #[error("rpc internal error: {0}")]
    RpcInternalError(#[from] rpc::RpcError),
    #[error("serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Process error (serialized): {0}")]
    SerializedErr(String),
}

pub async fn run(text: &str) -> String {
    match serde_json::from_str::<Cmd>(text) {
        Ok(cmd) => match process(cmd).await {
            Ok(rs) => rs,
            Err(err) => serialize_error(ProcessError::from(err), text),
        },
        Err(e) => serialize_error(ProcessError::SerdeError(e), text),
    }
}

pub fn run_sync(text: &str) -> String {
    let rt = Runtime::new().unwrap();
    rt.block_on(run(text))
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
        error: ProcessError::IoError(std::io::Error::new(
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
    eprintln!("process cmd {:?}", cmd);
    let pool = db::init_db().await?;

    let result = match cmd {
        Cmd::Server(s) => {
            crate::rpc::start(&s.addr, &pool).await?;
            Ok(serde_json::to_string(&ServerResponse {
                server: "started".to_string(),
            })?)
        }
        Cmd::ClientSync(s) => {
            let resp = crate::rpc::sync(&s.addr, &pool).await?;
            Ok(serde_json::to_string(&ClientSyncResponse {
                client_sync: resp,
            })?)
        }
        Cmd::ClientStopServer(s) => {
            let resp = crate::rpc::stop_server(&s.addr, &pool).await?;
            Ok(serde_json::to_string(&ClientStopServerResponse {
                client_stop_server: resp,
            })?)
        }
        Cmd::DbCmd(db_cmd) => Ok(db::process_cmd(db_cmd, &pool).await?),
    };

    result
}

fn run_async(text: &str) -> String {
    let rt = Runtime::new().unwrap();
    rt.block_on(run(text))
}
