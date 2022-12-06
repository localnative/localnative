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
use stream_cancel::Trigger;
use tarpc::server::incoming::Incoming;
use tarpc::server::Channel;
use tarpc::{context, server};
use tokio::runtime::Runtime;
use tokio_serde::formats::Bincode;

#[derive(Clone)]
struct LocalNativeServer(SocketAddr, Option<tokio::sync::mpsc::Sender<()>>);

impl LocalNative for LocalNativeServer {
    type IsVersionMatchFut = Ready<bool>;
    #[allow(clippy::wrong_self_convention)]
    fn is_version_match(self, _: context::Context, version: String) -> Self::IsVersionMatchFut {
        let conn = get_sqlite_connection();
        let meta_version = get_meta_version(&conn).unwrap_or_else(|_| "0.3.10".into());
        if version == meta_version {
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
        match diff_uuid4_to_server(&conn, candidates) {
            Ok(diff) => future::ready(diff),
            Err(err) => {
                println!("diff_uuid4_to_server error: {}", err);
                future::ready(Vec::new())
            }
        }
    }
    type DiffUuid4FromServerFut = Ready<Vec<String>>;
    fn diff_uuid4_from_server(
        self,
        _: context::Context,
        candidates: Vec<String>,
    ) -> Self::DiffUuid4FromServerFut {
        let conn = get_sqlite_connection();
        match diff_uuid4_from_server(&conn, candidates) {
            Ok(diff) => future::ready(diff),
            Err(err) => {
                println!("diff_uuid4_from_server error: {}", err);
                future::ready(Vec::new())
            }
        }
    }
    type SendNoteFut = Ready<bool>;
    fn send_note(self, _: context::Context, note: Note) -> Self::SendNoteFut {
        eprintln!("upsert note {:?}", note);
        let res = match insert(note) {
            Ok(_) => true,
            Err(err) => {
                eprintln!("insert note error: {:?}", err);
                false
            }
        };
        future::ready(res)
    }
    type ReceiveNoteFut = Ready<Note>;
    fn receive_note(self, _: context::Context, uuid4: String) -> Self::ReceiveNoteFut {
        eprintln!("receive note {:?}", uuid4);
        let conn = get_sqlite_connection();
        match get_note_by_uuid4(&conn, &uuid4) {
            Ok(note) => future::ready(note),
            Err(err) => {
                println!("receive note error: {}", err);
                future::ready(Note::default())
            }
        }
    }
    type StopFut = Ready<()>;
    #[allow(unreachable_code)]
    fn stop(self, _: context::Context) -> Self::StopFut {
        eprintln!("server stopping");
        if let Some(ref exit_tx) = self.1 {
            let _ = exit_tx.try_send(());
        } else {
            process::exit(0);
        }
        future::ready(())
    }
}

async fn start_server(addr: SocketAddr, exit_tx: tokio::sync::mpsc::Sender<()>) -> io::Result<()> {
    tarpc::serde_transport::tcp::listen(addr, Bincode::default)
        .await?
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 2 per IP.
        .max_channels_per_key(2, |t| t.as_ref().peer_addr().unwrap().ip())
        .map(move |channel| {
            let server = LocalNativeServer(
                channel.as_ref().as_ref().peer_addr().unwrap(),
                Some(exit_tx.clone()),
            );
            channel.execute(server.serve())
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}
use tokio::sync::oneshot::Receiver;
pub type Stop = Receiver<Trigger>;
pub async fn iced_start_server(addr: SocketAddr) -> io::Result<Stop> {
    use stream_cancel::Valved;
    let (exit_sender, exit_receiver) = tokio::sync::oneshot::channel();
    let listener = tarpc::serde_transport::tcp::listen(addr, Bincode::default).await?;

    tokio::spawn(async move {
        let (exit, incoming) = Valved::new(listener);
        exit_sender.send(exit).unwrap();
        incoming
            .filter_map(|r| future::ready(r.ok()))
            .map(server::BaseChannel::with_defaults)
            // Limit channels to 2 per IP.
            .max_channels_per_key(2, |t| t.as_ref().peer_addr().unwrap().ip())
            .map(|channel| {
                let server =
                    LocalNativeServer(channel.as_ref().as_ref().peer_addr().unwrap(), None);
                channel.execute(server.serve())
            })
            // Max 10 channels.
            .buffer_unordered(10)
            .for_each(|_| async {})
            .await
    });

    Ok(exit_receiver)
}

pub fn start(addr: &str) -> Result<(), &'static str> {
    let server_addr: SocketAddr = addr
        .parse()
        .unwrap_or_else(|e| panic!(r#"server_addr {} invalid: {}"#, addr, e));
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let (exit_tx, mut exit_rx) = tokio::sync::mpsc::channel(1);

        tokio::select! {
            _ = start_server(server_addr, exit_tx) => {},
            _ = exit_rx.recv() => {
                eprintln!("server task exit");
            }
        }
    });
    Ok(())
}

pub fn get_server_addr() -> String {
    for iface in get_if_addrs::get_if_addrs().unwrap() {
        if !iface.is_loopback() {
            return format!("{}:3456", iface.addr.ip());
        }
    }
    "".to_string()
}
