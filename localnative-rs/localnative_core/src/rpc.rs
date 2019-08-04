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
use crate::Note;

tarpc::service! {
    rpc is_version_match(version: String) -> bool;
    rpc diff_uuid4_to_server(candidates: Vec<String>) -> Vec<String> ;
    rpc diff_uuid4_from_server(candidates: Vec<String>) -> Vec<String> ;
    rpc send_note(note: Note) -> bool ;
    rpc receive_note(uuid4: String) -> Note ;
    rpc stop() -> bool ;
}

pub mod client;
pub mod server;
