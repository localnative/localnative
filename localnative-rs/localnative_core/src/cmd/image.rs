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
use crate::Note;
use base64::decode;
use rusqlite::types::ToSql;

pub fn insert_image(note: Note) -> anyhow::Result<()> {
    let data64 = note.annotations.replace("data:image/png;base64,", "");
    let decoded = decode(&data64)?;
    let conn = &mut super::super::exe::get_sqlite_connection();
    let tx = conn.transaction()?;
    {
        tx.execute(
            "
        INSERT INTO note (title, uuid4, url, tags, description, comments, annotations, created_at, is_public)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);

        ",
            &[
                &note.title,
                &note.uuid4,
                &note.url,
                &make_tags(&note.tags),
                &note.description,
                &note.comments,
                &decoded as &dyn ToSql,
                // &note.annotations,
                &note.created_at,
                &note.is_public as &dyn ToSql,
            ],
        )?;
    }
    {
        // mark is_last_note = 0 to indicate out of sync, i.e. db > ssb
        tx.execute(
            "
        UPDATE ssb SET is_last_note = 0
        WHERE is_active_author = 1
        ",
            [],
        )?;
    }
    tx.commit()?;
    Ok(())
}
