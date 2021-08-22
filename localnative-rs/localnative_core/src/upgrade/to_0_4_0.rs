/*
    Local Native
    Copyright (C) 2019  Yi Wang

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

use super::utils;
use super::uuid::Uuid;
use rusqlite::{Connection, ToSql};

// insert each record to note from _note_0_3 with newly generated uuid4 value
pub fn migrate_note(conn: &Connection) -> anyhow::Result<()> {
    eprintln!("to_0_4_0 migrate_note");
    if utils::check_table_exist(conn, "_note_0_3")? {
        eprintln!("to_0_4_0 _note_0_3 exists, looping each record");
        let mut stmt = conn.prepare(
            "SELECT rowid, title, url, tags, description, comments
        , annotations
        , created_at, is_public
        FROM _note_0_3
        order by rowid",
        )?;
        let note_iter = stmt.query_map([], |row| {
            Ok(Note {
                rowid: row.get(0)?,
                uuid4: Uuid::new_v4().to_string(),
                title: row.get(1)?,
                url: row.get(2)?,
                tags: row.get(3)?,
                description: row.get(4)?,
                comments: row.get(5)?,
                annotations: row.get(6)?,
                created_at: row.get(7)?,
                is_public: row.get(8)?,
            })
        })?;

        for note in note_iter {
            let note = note?;
            conn.execute(
                "INSERT INTO note (uuid4, title, url, tags, description, comments
        , annotations
        , created_at, is_public)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                &[
                    &note.uuid4,
                    &note.title,
                    &note.url,
                    &note.tags,
                    &note.description,
                    &note.comments,
                    &note.annotations,
                    &note.created_at,
                    &note.is_public as &dyn ToSql,
                ],
            )?;
        }
        eprintln!("to_0_4_0 drop _note_0_3");
        conn.execute_batch(
            "BEGIN;
        drop table _note_0_3;
        UPDATE meta SET meta_value = '0'
        WHERE meta_key = 'is_upgrading';
        COMMIT;",
        )?;
    }
    Ok(())
}

// rename note to _note_0_3
// create new note table with new uuid4 column
// create meta table
// set version 0.4.0
pub fn migrate_schema(conn: &Connection) -> anyhow::Result<()> {
    eprintln!("to_0_4_0 migrate_schema");
    conn.execute_batch(
        "BEGIN;
    ALTER TABLE note RENAME TO _note_0_3;
CREATE TABLE note (
rowid          INTEGER PRIMARY KEY AUTOINCREMENT,
uuid4          TEXT NOT NULL UNIQUE,
title          TEXT NOT NULL,
url            TEXT NOT NULL,
tags           TEXT NOT NULL,
description    TEXT NOT NULL,
comments       TEXT NOT NULL,
annotations    TEXT NOT NULL,
created_at     TEXT NOT NULL,
is_public      BOOLEAN NOT NULL default 0
);
INSERT INTO meta (
    meta_key,
    meta_value
)
VALUES
('version','0.4.0'),
('is_upgrading', '1');
COMMIt;",
    )?;
    Ok(())
}

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
