pub extern crate dirs;
pub extern crate rusqlite;
extern crate serde;
pub extern crate serde_json;
#[macro_use]
extern crate serde_derive;
pub mod cmd;
pub mod exe;

// JNI interface for android
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub mod android {
    extern crate jni;

    use self::jni::objects::{JClass, JString};
    use self::jni::sys::jstring;
    use self::jni::JNIEnv;
    use super::*;

    #[no_mangle]
    pub unsafe extern "C" fn Java_app_localnative_android_RustBridge_localnativeRun(
        env: JNIEnv,
        _: JClass,
        json_input: JString,
    ) -> jstring {
        let json = localnative_run(
            env.get_string(json_input)
                .expect("Invalid json input string!")
                .as_ptr(),
        );
        // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
        let output_ptr = CString::from_raw(json);
        let output = env
            .new_string(output_ptr.to_str().unwrap())
            .expect("Couldn't create java output string!");

        output.into_inner()
    }
}

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn localnative_run(json_input: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(json_input) };
    let json = match c_str.to_str() {
        Err(_) => r#"{"error": "ios json input error"}"#.to_string(),
        Ok(text) => exe::run(&text),
    };

    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn localnative_free(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        CString::from_raw(s)
    };
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ssbify {
    pub hash: String,
    pub markdown: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SsbNote {
    pub note_title: String,
    pub note_url: String,
    pub note_tags: String,
    pub note_description: String,
    pub note_comments: String,
    pub note_annotations: String,
    pub note_created_at: String,

    pub author: String,
    pub ts: i64,
    pub key: String,
    pub prev: String,
    pub seq: i64,
    pub is_public: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ssb {
    pub note_rowid: i64,
    pub author: String,
    pub is_active_author: bool,
    pub is_last_note: bool,
    pub ts: i64,
    pub seq: i64,
    pub key: String,
    pub prev: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    pub rowid: i64,
    pub title: String,
    pub url: String,
    pub tags: String,
    pub description: String,
    pub comments: String,
    pub annotations: String,
    pub created_at: String,
    pub is_public: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cmd {
    pub action: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdInsert {
    pub title: String,
    pub url: String,
    pub tags: String,
    pub description: String,
    pub comments: String,
    pub annotations: String,

    pub limit: u32,
    pub offset: u32,
    pub is_public: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdSyncViaAttach {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdSearch {
    pub query: String,

    pub limit: u32,
    pub offset: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdDelete {
    pub query: String,
    pub rowid: i64,

    pub limit: u32,
    pub offset: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdSelect {
    pub limit: u32,
    pub offset: u32,
}
