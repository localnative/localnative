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
        if version == "0.4.0" {
            future::ready(true)
        } else {
            future::ready(false)
        }
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
