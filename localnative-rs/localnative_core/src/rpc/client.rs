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
use super::LnClient;
use crate::cmd::insert;
use crate::cmd::sync::get_note_by_uuid4;
use crate::cmd::sync::next_uuid4_candidates;
use crate::exe::get_sqlite_connection;
use crate::upgrade::get_meta_version;
use std::io::{Error, ErrorKind};
use std::{io, net::SocketAddr};
use tarpc::{client, context};
use tokio::runtime::Runtime;
use tokio_serde::formats::Bincode;

async fn run_sync_to_server(addr: &SocketAddr) -> io::Result<()> {
    let transport = tarpc::serde_transport::tcp::connect(addr, Bincode::default).await?;
    let client = LnClient::new(client::Config::default(), transport).spawn()?;
    let conn = get_sqlite_connection();

    // check version
    let version = get_meta_version(&conn);
    let is_version_match = client.is_version_match(context::current(), version).await?;
    eprintln!("is_version_match: {}", is_version_match);
    if !is_version_match {
        return Err(Error::new(ErrorKind::Other, "VERSION_NOT_MATCH"));
    }

    // diff uuid4
    let diff_uuid4 = client
        .diff_uuid4_to_server(context::current(), next_uuid4_candidates(&conn))
        .await?;
    eprintln!("diff_uuid4_to_server len: {:?}", diff_uuid4.len());

    // send one by one
    for u in diff_uuid4 {
        client
            .send_note(context::current(), get_note_by_uuid4(&conn, &u))
            .await?;
    }
    eprintln!("send_note done");

    Ok(())
}

async fn run_sync_from_server(addr: &SocketAddr) -> io::Result<()> {
    let transport = tarpc::serde_transport::tcp::connect(addr, Bincode::default).await?;
    let client = LnClient::new(client::Config::default(), transport).spawn()?;
    let conn = get_sqlite_connection();

    // check version
    let version = get_meta_version(&conn);
    let is_version_match = client.is_version_match(context::current(), version).await?;
    eprintln!("is_version_match: {}", is_version_match);
    if !is_version_match {
        return Err(Error::new(ErrorKind::Other, "VERSION_NOT_MATCH"));
    }

    // diff uuid4
    let diff_uuid4 = client
        .diff_uuid4_from_server(context::current(), next_uuid4_candidates(&conn))
        .await?;
    eprintln!("diff_uuid4_from_server len: {:?}", diff_uuid4.len());

    // send one by one
    for u in diff_uuid4 {
        let note = client.receive_note(context::current(), u).await?;
        insert(note);
    }
    eprintln!("receive_note done");

    Ok(())
}

pub fn sync(addr: &str) -> Result<String, String> {
    let server_addr = addr
        .parse()
        .unwrap_or_else(|e| panic!(r#"server_addr {} invalid: {}"#, addr, e));
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        run_sync_to_server(&server_addr).await.unwrap();
        eprintln!("sync to server done");
    });
    let rt2 = Runtime::new().unwrap();
    rt2.block_on(async {
        run_sync_from_server(&server_addr).await.unwrap();
        eprintln!("sync from server done");
    });
    Ok("sync ok".to_string())
}

pub async fn run_stop_server(addr: &SocketAddr) -> io::Result<()> {
    let transport = tarpc::serde_transport::tcp::connect(addr, Bincode::default).await?;
    let client = LnClient::new(client::Config::default(), transport).spawn()?;
    let conn = get_sqlite_connection();

    // check version
    let version = get_meta_version(&conn);
    let is_version_match = client.is_version_match(context::current(), version).await?;
    eprintln!("is_version_match: {}", is_version_match);
    if !is_version_match {
        return Err(Error::new(ErrorKind::Other, "VERSION_NOT_MATCH"));
    }

    // diff uuid4
    client.stop(context::current()).await?;
    Ok(())
}

pub fn stop_server(addr: &str) -> Result<String, String> {
    let server_addr: SocketAddr = addr
        .parse()
        .unwrap_or_else(|e| panic!(r#"server_addr {} invalid: {}"#, addr, e));
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        run_stop_server(&server_addr).await.unwrap();
    });
    Ok("stop ok".to_string())
}
