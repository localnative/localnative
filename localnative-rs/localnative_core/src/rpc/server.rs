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
use crate::cmd::sync::{diff_uuid4_from_server, diff_uuid4_to_server};
use crate::exe::get_sqlite_connection;
use crate::upgrade::get_meta_version;
use crate::Note;
use std::process;

use futures::{
    compat::Executor01CompatExt,
    future::{self, Ready},
    prelude::*,
};
use std::{io, net::SocketAddr};
use tarpc::{
    context,
    server::{Handler, Server},
};

#[derive(Clone)]
struct LocalNativeServer;

impl super::Service for LocalNativeServer {
    type IsVersionMatchFut = Ready<bool>;
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
    type ReceiveNoteFut = Ready<bool>;
    fn receive_note(self, _: context::Context, note: Note) -> Self::ReceiveNoteFut {
        eprintln!("upsert note {:?}", note);
        insert(note);
        future::ready(true)
    }
    type StopFut = Ready<bool>;
    fn stop(self, _: context::Context) -> Self::StopFut {
        eprintln!("server stopping");
        process::exit(0);
        future::ready(true)
    }
}

async fn localnative_server(addr: SocketAddr) -> io::Result<()> {
    let transport = bincode_transport::listen(&addr)?;
    let server = Server::default()
        .incoming(transport)
        .respond_with(super::serve(LocalNativeServer));
    server.await;
    Ok(())
}

pub fn start(addr: &str) -> Result<(), &'static str> {
    let server_addr = addr
        .parse()
        .unwrap_or_else(|e| panic!(r#"server_addr {} invalid: {}"#, addr, e));
    tarpc::init(tokio::executor::DefaultExecutor::current().compat());
    tokio::run(
        localnative_server(server_addr)
            .map_err(|err| eprintln!("localnative server error: {}", err))
            .boxed()
            .compat(),
    );
    Ok(())
}
