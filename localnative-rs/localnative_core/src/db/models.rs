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

//! Request and response types of the JSON database commands.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Day {
    pub date: NaiveDate,
    pub count: i64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Tags {
    pub tag: String,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Note {
    pub rowid: i64,
    pub uuid4: String,
    pub title: String,
    pub url: String,
    pub tags: String,
    pub description: String,
    pub comments: String,
    /// The note's annotations as the write side accepts them: UTF-8 text as
    /// is, and binary content (screenshots) as a `data:` URL.
    pub annotations: String,
    pub created_at: String,
    pub is_public: bool,
    pub metadata: String,
    /// Last-write-wins token (see [`crate::db`]); read paths may leave it empty.
    #[serde(default)]
    pub updated_at: String,
    /// Tombstone flag. Read queries never return tombstones.
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum Cmd {
    Insert(CmdInsert),
    InsertImage(CmdInsert),
    Delete(CmdDelete),
    Select(CmdSelect),
    Search(CmdSearch),
    Filter(CmdFilter),
    Upgrade,
    SyncViaAttach(CmdSyncViaAttach),
    ExportDb(CmdExportDb),
    ImportDb(CmdImportDb),
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
pub struct CmdFilter {
    pub query: String,
    pub limit: u32,
    pub offset: u32,
    pub from: String,
    pub to: String,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdSyncViaAttach {
    pub uri: String,
}

/// Export a clean, single-file copy of the database to `dest`.
#[derive(Serialize, Deserialize, Debug)]
pub struct CmdExportDb {
    pub dest: String,
}

/// Import (one-way LWW merge) notes from another database file at `src`.
#[derive(Serialize, Deserialize, Debug)]
pub struct CmdImportDb {
    pub src: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct QueryResult {
    pub count: u32,
    pub notes: Vec<Note>,
    pub days: Vec<Day>,
    pub tags: Vec<Tags>,
}
