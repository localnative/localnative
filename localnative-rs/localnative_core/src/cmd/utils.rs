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
use base64::Engine;
use hex::FromHex;
pub fn make_data_url(row: &rusqlite::Row) -> anyhow::Result<String> {
    let url = row.get::<_, String>(3)?;
    #[cfg(not(feature = "no_print"))]
    eprintln!("url: {}", url);
    if url == "mime://image/png" {
        let hex = row.get::<_, String>(7)?;
        let result = base64::prelude::BASE64_STANDARD.encode(Vec::<u8>::from_hex(hex)?);
        let mut r = "data:image/png;base64,".to_owned();
        r.push_str(&result);
        Ok(r)
    } else {
        Ok("".to_string())
    }
}
