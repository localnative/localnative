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
use super::make_tags;
use crate::{KVStringI64, Note, Tags};
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use std::collections::HashMap;

pub fn next_uuid4_candidates(conn: &Connection) -> Vec<String> {
    vec!["1".to_string(), "2".to_string(), "3".to_string()]
}

pub fn diff_uuid4(conn: &Connection, candidates: Vec<String>) -> Vec<String> {
    let mut r = Vec::new();
    let mut stmt = conn.prepare("select 1 FROM note where uuid4 = ? ").unwrap();
    for uuid4 in candidates {
        if !(stmt.exists(&[&uuid4]).unwrap()) {
            r.push(uuid4);
        }
    }
    r
}
