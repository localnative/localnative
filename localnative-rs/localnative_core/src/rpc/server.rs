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

use crate::cmd::insert;
use crate::cmd::sync::{diff_uuid4_from_server, diff_uuid4_to_server, get_note_by_uuid4};
use crate::exe::get_sqlite_connection;
use crate::upgrade::get_meta_version;
use crate::Note;
use std::process;

use super::LocalNative;
use futures::{
    future::{self, Ready},
    prelude::*,
};
use std::{io, net::SocketAddr};
use tarpc::{
    context,
    server::{self, Channel, Incoming},
};
use tokio::runtime::Runtime;
use tokio_serde::formats::Bincode;

#[derive(Clone)]
struct LocalNativeServer(SocketAddr);

impl LocalNative for LocalNativeServer {
    type IsVersionMatchFut = Ready<bool>;
    #[allow(clippy::wrong_self_convention)]
    fn is_version_match(self, _: context::Context, version: String) -> Self::IsVersionMatchFut {
        let conn = get_sqlite_connection();
        if version == get_meta_version(&conn) {
            future::ready(true)
        } else {
            future::ready(false)
        }
    }
    type DiffUuid4ToServerFut = Ready<Vec<String>>;
    fn diff_uuid4_to_server(
        self,
        _: context::Context,
        candidates: Vec<String>,
    ) -> Self::DiffUuid4ToServerFut {
        let conn = get_sqlite_connection();
        let diff = diff_uuid4_to_server(&conn, candidates);
        future::ready(diff)
    }
    type DiffUuid4FromServerFut = Ready<Vec<String>>;
    fn diff_uuid4_from_server(
        self,
        _: context::Context,
        candidates: Vec<String>,
    ) -> Self::DiffUuid4FromServerFut {
        let conn = get_sqlite_connection();
        let diff = diff_uuid4_from_server(&conn, candidates);
        future::ready(diff)
    }
    type SendNoteFut = Ready<bool>;
    fn send_note(self, _: context::Context, note: Note) -> Self::SendNoteFut {
        eprintln!("upsert note {:?}", note);
        insert(note);
        future::ready(true)
    }
    type ReceiveNoteFut = Ready<Note>;
    fn receive_note(self, _: context::Context, uuid4: String) -> Self::ReceiveNoteFut {
        eprintln!("receive note {:?}", uuid4);
        let conn = get_sqlite_connection();
        let note = get_note_by_uuid4(&conn, &uuid4);
        future::ready(note)
    }
    type StopFut = Ready<()>;
    #[allow(unreachable_code)]
    fn stop(self, _: context::Context) -> Self::StopFut {
        eprintln!("server stopping");
        process::exit(0);
        future::ready(())
    }
}

pub async fn start_server(addr: &SocketAddr) -> io::Result<()> {
    tarpc::serde_transport::tcp::listen(addr, Bincode::default)
        .await?
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(1, |t| t.as_ref().peer_addr().unwrap().ip())
        .map(|channel| {
            let server = LocalNativeServer(channel.as_ref().as_ref().peer_addr().unwrap());
            channel.execute(server.serve())
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}

pub fn start(addr: &str) -> Result<(), &'static str> {
    let server_addr: SocketAddr = addr
        .parse()
        .unwrap_or_else(|e| panic!(r#"server_addr {} invalid: {}"#, addr, e));
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        start_server(&server_addr).await.unwrap();
    });
    Ok(())
}

pub fn get_server_addr() -> String {
    for iface in get_if_addrs::get_if_addrs().unwrap() {
        if !iface.is_loopback() {
            return format!("{}:3456", iface.addr.ip().to_string());
        }
    }
    "".to_string()
}
