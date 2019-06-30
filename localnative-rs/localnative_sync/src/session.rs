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

pub struct SyncSession {
    pub token: String,
    pub url: String,
    pub created_at: String,
}

pub struct BatchIdsSyncSession {
    pub uuid: String,
    pub url: String,
    pub created_at: String,
}

pub fn open_sync_session() {}

pub fn close_sync_session() {}

pub fn send_batch() {}

pub fn receive_batch() {}

pub fn ack_batch() {}
