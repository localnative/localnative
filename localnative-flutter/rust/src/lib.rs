use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Import the existing C FFI functions from localnative_core
extern "C" {
    fn localnative_run(json_input: *const c_char) -> *mut c_char;
    fn localnative_free(s: *mut c_char);
}

/// Safe wrapper around the localnative_run C function
fn run_command(json: &str) -> Result<String, String> {
    let c_str = CString::new(json).map_err(|e| e.to_string())?;

    unsafe {
        let result_ptr = localnative_run(c_str.as_ptr());
        if result_ptr.is_null() {
            return Err("Null pointer returned from localnative_run".to_string());
        }

        let result_cstr = CStr::from_ptr(result_ptr);
        let result = result_cstr.to_string_lossy().into_owned();
        localnative_free(result_ptr);

        Ok(result)
    }
}

// Data models matching the Rust core

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub rowid: i64,
    pub uuid4: String,
    pub title: String,
    pub url: String,
    pub tags: String,
    pub description: String,
    pub comments: String,
    pub annotations: String,
    pub created_at: String,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayCount {
    pub k: String,  // Date
    pub v: i64,     // Count
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCount {
    pub k: String,  // Tag name
    pub v: i64,     // Count
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesResponse {
    pub count: i64,
    pub notes: Vec<Note>,
    pub days: Vec<DayCount>,
    pub tags: Vec<TagCount>,
}

// Command builders

#[frb(sync)]
pub fn select_notes(limit: i64, offset: i64) -> Result<NotesResponse, String> {
    let cmd = serde_json::json!({
        "action": "select",
        "limit": limit,
        "offset": offset
    });

    let result = run_command(&cmd.to_string())?;
    serde_json::from_str(&result).map_err(|e| e.to_string())
}

#[frb(sync)]
pub fn search_notes(query: String, limit: i64, offset: i64) -> Result<NotesResponse, String> {
    let cmd = serde_json::json!({
        "action": "search",
        "query": query,
        "limit": limit,
        "offset": offset
    });

    let result = run_command(&cmd.to_string())?;
    serde_json::from_str(&result).map_err(|e| e.to_string())
}

#[frb(sync)]
pub fn insert_note(
    title: String,
    url: String,
    tags: String,
    description: String,
    comments: String,
    annotations: String,
    is_public: bool,
    limit: i64,
    offset: i64,
) -> Result<NotesResponse, String> {
    let cmd = serde_json::json!({
        "action": "insert",
        "title": title,
        "url": url,
        "tags": tags,
        "description": description,
        "comments": comments,
        "annotations": annotations,
        "is_public": is_public,
        "limit": limit,
        "offset": offset
    });

    let result = run_command(&cmd.to_string())?;
    serde_json::from_str(&result).map_err(|e| e.to_string())
}

#[frb(sync)]
pub fn delete_note(rowid: i64, limit: i64, offset: i64) -> Result<NotesResponse, String> {
    let cmd = serde_json::json!({
        "action": "delete",
        "query": "",
        "rowid": rowid,
        "limit": limit,
        "offset": offset
    });

    let result = run_command(&cmd.to_string())?;
    serde_json::from_str(&result).map_err(|e| e.to_string())
}

#[frb(sync)]
pub fn filter_notes(
    query: String,
    from: String,
    to: String,
    limit: i64,
    offset: i64,
) -> Result<NotesResponse, String> {
    let cmd = serde_json::json!({
        "action": "filter",
        "query": query,
        "from": from,
        "to": to,
        "limit": limit,
        "offset": offset
    });

    let result = run_command(&cmd.to_string())?;
    serde_json::from_str(&result).map_err(|e| e.to_string())
}

#[frb(sync)]
pub fn start_server(addr: String) -> Result<String, String> {
    let cmd = serde_json::json!({
        "action": "server",
        "addr": addr
    });

    run_command(&cmd.to_string())
}

#[frb(sync)]
pub fn sync_with_server(addr: String) -> Result<String, String> {
    let cmd = serde_json::json!({
        "action": "client-sync",
        "addr": addr
    });

    run_command(&cmd.to_string())
}

#[frb(sync)]
pub fn stop_server(addr: String) -> Result<String, String> {
    let cmd = serde_json::json!({
        "action": "client-stop-server",
        "addr": addr
    });

    run_command(&cmd.to_string())
}

#[frb(sync)]
pub fn sync_via_attach(db_path: String, limit: i64, offset: i64) -> Result<NotesResponse, String> {
    let cmd = serde_json::json!({
        "action": "sync-via-attach",
        "db_path": db_path,
        "limit": limit,
        "offset": offset
    });

    let result = run_command(&cmd.to_string())?;
    serde_json::from_str(&result).map_err(|e| e.to_string())
}

// Initialize the library
#[frb(init)]
pub fn init_app() {
    // Any initialization logic if needed
    flutter_rust_bridge::setup_default_user_utils();
}
