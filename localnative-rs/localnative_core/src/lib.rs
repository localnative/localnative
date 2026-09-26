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
//! Local Native core: one Rust library behind every front-end.
//!
//! Front-ends serialize a command to JSON, hand it to [`run`] (or
//! [`localnative_run`] across the FFI), and get JSON back. Every failure —
//! parse, database, sync — comes back as `{"error": <message>, "code": <code>}`;
//! there is one error shape, and it never echoes the request.

use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::net::SocketAddr;
use std::os::raw::c_char;
use std::sync::OnceLock;
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
pub mod secure;
pub mod wire;

// Re-export error types at crate root for convenience.
pub use error::{DatabaseError, Error, ProcessError, SyncError, ValidationError};

#[cfg(target_os = "android")]
pub mod android {
    use super::*;
    use jni::EnvUnowned;
    use jni::errors::ThrowRuntimeExAndDefault;
    use jni::objects::{JClass, JString};

    // jni 0.22 split `JNIEnv` into `Env` (full API) and `EnvUnowned` (FFI-safe).
    // Native methods receive `EnvUnowned`; `with_env` upgrades it to an `Env`,
    // wraps the body in `catch_unwind` (so a panic can't unwind into the JVM),
    // and `resolve` returns the value or throws a RuntimeException + the default
    // (a null `JString`) on error/panic.
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_app_localnative_android_RustBridge_localnativeRun<'local>(
        mut env: EnvUnowned<'local>,
        _class: JClass<'local>,
        json_input: JString<'local>,
    ) -> JString<'local> {
        env.with_env(|env| -> jni::errors::Result<JString<'local>> {
            let json = json_input.mutf8_chars(env)?.to_str().into_owned();
            let result = run_async(&json);
            env.new_string(result)
        })
        .resolve::<ThrowRuntimeExAndDefault>()
    }

    /// Point the core at this app's own database directory. Must be called
    /// before [`Java_app_localnative_android_RustBridge_localnativeRun`].
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_app_localnative_android_RustBridge_localnativeSetDbPath<'local>(
        mut env: EnvUnowned<'local>,
        _class: JClass<'local>,
        path: JString<'local>,
    ) {
        let _ = env.with_env(|env| -> jni::errors::Result<()> {
            let path = path.mutf8_chars(env)?.to_str().into_owned();
            db::set_db_path(path);
            Ok(())
        });
    }
}

/// # Safety
///
/// `json_input` must be a valid, non-null pointer to a nul-terminated C string that remains
/// valid for the duration of this call. The returned pointer must be freed with
/// [`localnative_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn localnative_run(json_input: *const c_char) -> *mut c_char {
    unsafe {
        let c_str = CStr::from_ptr(json_input);
        let json = match c_str.to_str() {
            Ok(s) => run_async(s),
            Err(_) => r#"{"error":"Invalid UTF-8 in input","code":"invalid-input"}"#.to_string(),
        };

        match CString::new(json) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => CString::new(r#"{"error":"Response contained null byte","code":"internal"}"#)
                .unwrap_or_default()
                .into_raw(),
        }
    }
}

/// # Safety
///
/// `path` must be a valid, non-null pointer to a nul-terminated C string that
/// remains valid for the duration of this call. The string is copied.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn localnative_set_db_path(path: *const c_char) {
    unsafe {
        if let Ok(path) = CStr::from_ptr(path).to_str() {
            db::set_db_path(path);
        }
    }
}

/// # Safety
///
/// `s` must be a pointer previously returned by [`localnative_run`], or null. After this call
/// the pointer is invalid and must not be used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn localnative_free(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            drop(CString::from_raw(s));
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum Cmd {
    /// Start the sync server. `pairing: true` also accepts one new device.
    Server(CmdServer),
    /// Start (or refresh) server pairing; the reply carries the code to show.
    ServerPairing(CmdAddr),
    /// Stop the sync server this process is running (local action).
    ServerStop(CmdAddr),
    /// Sync with a peer; `code` pairs on first contact.
    ClientSync(CmdClientSync),
    /// List paired devices.
    Peers,
    /// Unpair a device.
    PeerForget(CmdPeerForget),
    #[serde(untagged)]
    DbCmd(db::models::Cmd),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdServer {
    pub addr: String,
    #[serde(default)]
    pub pairing: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdAddr {
    pub addr: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdClientSync {
    pub addr: String,
    #[serde(default)]
    pub code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdPeerForget {
    pub public_key: String,
}

/// Parse `text` as a JSON [`Cmd`], execute it, and return the result as a JSON string.
/// Errors are serialized into the returned string rather than propagated.
pub async fn run(text: &str) -> String {
    match serde_json::from_str::<Cmd>(text) {
        Ok(cmd) => match process(cmd).await {
            Ok(rs) => rs,
            Err(err) => serialize_error(&err),
        },
        Err(e) => serialize_error(&ProcessError::Serde(e)),
    }
}

/// Synchronous wrapper around [`run`] — blocks the current thread on the shared Tokio runtime.
/// Intended for use from C FFI and other non-async callers.
pub fn run_sync(text: &str) -> String {
    global_runtime().block_on(run(text))
}

/// The one error envelope every front-end can rely on: a message for humans
/// and a stable code for machines.
fn serialize_error(err: &ProcessError) -> String {
    serde_json::json!({ "error": err.to_string(), "code": err.code() }).to_string()
}

async fn process(cmd: Cmd) -> Result<String, ProcessError> {
    tracing::debug!(?cmd, "processing command");

    match cmd {
        Cmd::Server(s) => {
            let pool = db::init_pool()?;
            if s.pairing {
                let code = rpc::start_pairing(&s.addr, &pool).await?;
                let addresses = rpc::server_addresses(s.addr.parse::<SocketAddr>()?.port());
                Ok(
                    serde_json::json!({ "server": "started", "pairing-code": code, "addresses": addresses })
                        .to_string(),
                )
            } else {
                rpc::start(&s.addr, &pool).await?;
                Ok(serde_json::json!({ "server": "started" }).to_string())
            }
        }
        Cmd::ServerPairing(s) => {
            let pool = db::init_pool()?;
            let code = rpc::start_pairing(&s.addr, &pool).await?;
            let addresses = rpc::server_addresses(s.addr.parse::<SocketAddr>()?.port());
            Ok(
                serde_json::json!({ "server": "started", "pairing-code": code, "addresses": addresses })
                    .to_string(),
            )
        }
        Cmd::ServerStop(s) => {
            let stopped = rpc::stop_local(&s.addr).await?;
            Ok(serde_json::json!({ "server": stopped }).to_string())
        }
        Cmd::ClientSync(s) => {
            let pool = db::init_pool()?;
            let resp = rpc::sync(&s.addr, s.code.as_deref(), &pool).await?;
            Ok(serde_json::json!({ "client-sync": resp }).to_string())
        }
        Cmd::Peers => {
            let conn = db::init_db()?;
            Ok(serde_json::json!({ "peers": db::peers::list(&conn)? }).to_string())
        }
        Cmd::PeerForget(p) => {
            let conn = db::init_db()?;
            let forgotten = db::peers::forget(&conn, &p.public_key)?;
            Ok(serde_json::json!({ "peer-forgotten": forgotten }).to_string())
        }
        Cmd::DbCmd(db_cmd) => {
            let conn = db::init_db()?;
            Ok(db::process_cmd(db_cmd, &conn)?)
        }
    }
}

fn run_async(text: &str) -> String {
    global_runtime().block_on(run(text))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let cmd: Cmd = serde_json::from_str(&json).unwrap();
        assert!(matches!(cmd, Cmd::DbCmd(_)));

        let cmd = Cmd::DbCmd(db::models::Cmd::Search(db::models::CmdSearch {
            query: "hello".into(),
            limit: 10,
            offset: 0,
        }));
        let json = serde_json::to_string_pretty(&cmd).expect("Failed to serialize command");
        let cmd: Cmd = serde_json::from_str(&json).unwrap();
        assert!(matches!(cmd, Cmd::DbCmd(_)));
    }

    #[test]
    fn test_export_db_command_parses() {
        // The exact JSON the Android RustBridge (and any front-end) sends must
        // dispatch to the db ExportDb command via the untagged DbCmd fallthrough.
        let json = r#"{"action":"export-db","dest":"/storage/emulated/0/Android/data/app.localnative/files/localnative-export.sqlite3"}"#;
        let cmd = serde_json::from_str::<Cmd>(json).expect("export-db command should parse");
        match cmd {
            Cmd::DbCmd(db::models::Cmd::ExportDb(export)) => {
                assert!(export.dest.ends_with("localnative-export.sqlite3"));
            }
            other => panic!("expected DbCmd(ExportDb), got {other:?}"),
        }
    }

    #[test]
    fn test_import_db_command_parses() {
        let json = r#"{"action":"import-db","src":"/tmp/localnative-backup.sqlite3"}"#;
        let cmd = serde_json::from_str::<Cmd>(json).expect("import-db command should parse");
        match cmd {
            Cmd::DbCmd(db::models::Cmd::ImportDb(import)) => {
                assert_eq!(import.src, "/tmp/localnative-backup.sqlite3");
            }
            other => panic!("expected DbCmd(ImportDb), got {other:?}"),
        }
    }

    #[test]
    fn test_sync_commands_parse() {
        // Legacy payload without a code still parses; the error arrives later.
        let cmd =
            serde_json::from_str::<Cmd>(r#"{"action":"client-sync","addr":"127.0.0.1:2345"}"#)
                .expect("client-sync without code should parse");
        match cmd {
            Cmd::ClientSync(s) => assert_eq!(s.code, None),
            other => panic!("expected ClientSync, got {other:?}"),
        }

        let cmd = serde_json::from_str::<Cmd>(
            r#"{"action":"client-sync","addr":"127.0.0.1:2345","code":"abcd-efgh-jkmn-pqrs"}"#,
        )
        .expect("client-sync with code should parse");
        match cmd {
            Cmd::ClientSync(s) => {
                assert_eq!(s.code.as_deref(), Some("abcd-efgh-jkmn-pqrs"))
            }
            other => panic!("expected ClientSync, got {other:?}"),
        }

        let cmd =
            serde_json::from_str::<Cmd>(r#"{"action":"server-pairing","addr":"0.0.0.0:2345"}"#)
                .expect("server-pairing should parse");
        assert!(matches!(cmd, Cmd::ServerPairing(_)));

        let cmd = serde_json::from_str::<Cmd>(r#"{"action":"peers"}"#).expect("peers should parse");
        assert!(matches!(cmd, Cmd::Peers));
    }

    #[test]
    fn test_error_envelope() {
        let json = run_sync(r#"{"action":"no-such-action"}"#);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("error").is_some(), "envelope: {json}");
        assert!(parsed.get("code").is_some(), "envelope: {json}");
        // The failing command is not echoed back.
        assert!(parsed.get("source_text").is_none(), "envelope: {json}");
    }

    #[test]
    fn test_error_codes_are_stable() {
        for (error, code) in [
            (ProcessError::from(SyncError::NotPaired), "not-paired"),
            (
                ProcessError::from(SyncError::InvalidPairingCode),
                "invalid-pairing-code",
            ),
            (
                ProcessError::from(DatabaseError::Validation(ValidationError::InvalidPath(
                    "x".to_string(),
                ))),
                "invalid-input",
            ),
            (
                ProcessError::from(DatabaseError::EncryptionUnavailable),
                "encryption-unavailable",
            ),
        ] {
            let json = serialize_error(&error);
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed["code"].as_str(), Some(code), "{json}");
            assert!(parsed["error"].is_string(), "{json}");
        }
    }
}
