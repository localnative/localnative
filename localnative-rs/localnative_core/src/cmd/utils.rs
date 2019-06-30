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
// extern crate base64;
extern crate rusqlite;
// use self::base64::{decode, encode};
extern crate rustc_serialize;
use self::rustc_serialize::base64::{self, ToBase64};
use self::rustc_serialize::hex::FromHex;

pub fn make_data_url(row: &rusqlite::Row) -> String {
    let url = row.get::<_, String>(2).unwrap();
    eprintln!("url: {}", url);
    if url == "mime://image/png" {
        let hex = row.get::<_, String>(6).unwrap();
        let result = hex
            .from_hex()
            .unwrap()
            .as_slice()
            .to_base64(base64::STANDARD);
        let mut r = "data:image/png;base64,".to_owned();
        r.push_str(&result);
        r
    } else {
        "".to_string()
    }
}
