extern crate serde;
extern crate serde_json;
extern crate time;

#[macro_use]
extern crate serde_derive;

pub mod cmd;
pub mod exe;
pub mod sql;
pub mod ssb;

#[derive(Serialize, Deserialize, Debug)]
pub struct Ssb {
    pub note_rowid: i64,
    pub author: String,
    pub is_active_author: bool,
    pub is_last_note: bool,
    pub ts: String,
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
