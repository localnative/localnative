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

use futures::{compat::Executor01CompatExt, prelude::*};
use std::{io, net::SocketAddr};
use tarpc::{client, context};

async fn call_is_version_match(addr: SocketAddr, version: String) -> io::Result<()> {
    let transport = bincode_transport::connect(&addr).await?;
    let mut client = super::new_stub(client::Config::default(), transport).await?;
    let response = client.is_version_match(context::current(), version).await?;
    eprintln!("call_is_version_match: {}", response);
    Ok(())
}

pub fn is_version_match(addr: &str) -> Result<bool, &'static str> {
    tarpc::init(tokio::executor::DefaultExecutor::current().compat());

    let server_addr = addr
        .parse()
        .unwrap_or_else(|e| panic!(r#"server_addr {} invalid: {}"#, addr, e));

    let version = "0.4.0";

    tarpc::init(tokio::executor::DefaultExecutor::current().compat());

    tokio::run(
        call_is_version_match(server_addr, version.into())
            .map_err(|err| eprintln!("localnative client error: {}", err))
            .boxed()
            .compat(),
    );
    Ok(true)
}
