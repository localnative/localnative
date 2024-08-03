use iced::widget::Text;
use iced::widget::{
    button, column, horizontal_space, qr_code, row, text, text_input, tooltip, QRCode,
};
use iced::Command;
use iced::Element;
use iced_aw::NumberInput;

use once_cell::sync::OnceCell;
use ouroboros::self_referencing;
use regex::RegexSet;
use std::borrow::Cow;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

use std::{
    net::{Ipv6Addr, SocketAddr},
    path::PathBuf,
};

use localnative_core::rpc::server::Stop;
use tinyfiledialogs::open_file_dialog;

use crate::{
    tr,
    translate::{self, TranslateWithArgs},
    Conn,
};

use crate::{error_handle, icons::IconItem};

use self::ouroboros_impl_sync_view::Heads;

#[self_referencing]
pub struct SyncView {
    ip: String,
    pub port: u16,
    pub server_addr: String,
    pub ip_qr_code: qr_code::Data,
    pub sync_state: SyncState,
    pub server_state: ServerState,
    pub stop: Option<Stop>,
    #[borrows(server_addr)]
    #[covariant]
    pub translate: TranslateWithArgs<'this>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SyncState {
    Waiting,
    Syncing,
    SyncError(std::io::ErrorKind),
    Complete,
    IpAddrParseError,
    IpAddrParsePass,
    FilePathGetError,
    SyncFromFileUnknownError,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ServerState {
    Closed,
    Starting,
    Opened,
    Closing,
    Error,
}

#[derive(Debug, Clone)]
pub enum Message {
    IpInput(String),
    PortInput(u16),
    ClearAddrInput,
    SyncToServer,
    SyncFromServer,
    SyncFromFile,
    IpAddrVerify,
    OpenServer,
    Waiting,
    CloseServer,
}

impl SyncView {
    pub fn update_server_addr(&mut self, server_addr: String) {
        let old = core::mem::take(self);
        *self = Self::inner_update_server_addr(old, server_addr);
    }

    fn inner_update_server_addr(self, server_addr: String) -> Self {
        let Heads {
            stop,
            server_state,
            sync_state,
            ip_qr_code,
            port,
            ip,
            ..
        } = self.into_heads();

        SyncViewBuilder {
            stop,
            server_state,
            sync_state,
            ip_qr_code,
            server_addr,
            port,
            ip,
            translate_builder: |server_addr: &String| {
                translate::TranslateWithArgs::new("ip-qr", translate::args("ip", server_addr))
            },
        }
        .build()
    }

    pub fn view(&self) -> Element<Message> {
        let ip_input = text_input("xxx.xxx.xxx.xxx", self.borrow_ip())
            .on_input(Message::IpInput)
            .padding(0)
            .on_submit(Message::IpAddrVerify);

        let ip_tip = tooltip(
            ip_input,
            r"输入格式:xxx.xxx.xxx.xxx",
            iced::widget::tooltip::Position::Bottom,
        );

        let port_input = NumberInput::new(*self.borrow_port(), u16::MAX, Message::PortInput)
            .padding(0.)
            .on_submit(Message::IpAddrVerify);

        let clear_button = button(IconItem::Clear)
            .padding(0)
            .on_press(Message::ClearAddrInput);

        let ip_input_row = row![
            horizontal_space(),
            text(tr!("input-ip")),
            ip_tip,
            text(":"),
            port_input,
            clear_button,
            horizontal_space()
        ];

        let sync_from_server_button = button(row![
            IconItem::SyncFromServer,
            text(tr!("sync-from-server"))
        ])
        .padding(0)
        .on_press(Message::SyncFromServer);

        let sync_to_server_button =
            button(row![IconItem::SyncToServer, text(tr!("sync-to-server"))])
                .padding(0)
                .on_press(Message::SyncToServer);

        let sync_form_file_button =
            button(row![IconItem::SyncFromFile, text(tr!("sync-from-file"))])
                .padding(0)
                .on_press(Message::SyncFromFile);

        let content_text = match self.borrow_sync_state() {
            SyncState::Waiting => tr!("sync-waiting"),
            SyncState::Syncing => tr!("sync-syncing"),
            SyncState::SyncError(err) => {
                let prefix = tr!("sync-error").to_string();
                Cow::from(format!("{}{:?}", prefix, err))
            }
            SyncState::Complete => tr!("sync-complete"),
            SyncState::IpAddrParseError => tr!("sync-ip-parse-error"),
            SyncState::IpAddrParsePass => tr!("sync-ip-parse-complete"),
            SyncState::FilePathGetError => tr!("sync-file-path-error"),
            SyncState::SyncFromFileUnknownError => tr!("sync-from-file-unknown-error"),
        };

        let server_button_text = match self.borrow_server_state() {
            ServerState::Closed => row![IconItem::CloseServer, text(tr!("closed"))],
            ServerState::Starting => row![IconItem::Sync, text(tr!("starting"))],
            ServerState::Opened => row![IconItem::OpenServer, text(tr!("opened"))],
            ServerState::Closing => row![IconItem::Sync, text(tr!("closing"))],
            ServerState::Error => row![IconItem::Clear, text(tr!("unknow-error"))],
        };
        let mut server_button = button(server_button_text).padding(0);

        server_button = match self.borrow_server_state() {
            ServerState::Closed => server_button.on_press(Message::OpenServer),
            ServerState::Starting | ServerState::Closing | ServerState::Error => {
                server_button.on_press(Message::Waiting)
            }
            ServerState::Opened => server_button.on_press(Message::CloseServer),
        };

        let mut res = column![
            text(content_text),
            text(tr!("sync-client-tip")),
            text(tr!("input-ip-tip")),
            ip_input_row,
            row![
                sync_from_server_button,
                sync_to_server_button,
                sync_form_file_button
            ]
            .spacing(20)
            .align_items(iced::Alignment::Center),
            text(tr!("sync-server-tip")),
            server_button
        ]
        .spacing(20)
        .align_items(iced::Alignment::Center);

        if *self.borrow_server_state() == ServerState::Opened {
            res = res
                .push(Text::new(self.borrow_translate().tr()))
                .push(QRCode::new(self.borrow_ip_qr_code()));
        }

        res.into()
    }

    pub fn update(&mut self, message: Message, conn: Conn) -> Command<crate::Message> {
        match message {
            Message::IpInput(input) => {
                let ip_regex = IP_REGEX_SET.get_or_init(||{
                    RegexSet::new(&[
                        r"^$",
                        r"^(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.?$",
                        r"^(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.?$",
                        r"^(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.?$",
                        r"^(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|[0-4]\d|[0-1]?\d?\d)$",
                    ]).unwrap()
                });
                if ip_regex.is_match(&input) || Ipv6Addr::from_str(&input).is_ok() {
                    self.with_ip_mut(|ip| *ip = input);
                }
            }
            Message::PortInput(input) => {
                self.with_port_mut(|port| *port = input);
            }
            Message::SyncFromServer => {
                if let Ok(ip) = IpAddr::from_str(self.borrow_ip()) {
                    let addr = SocketAddr::new(ip, *self.borrow_port());
                    self.with_sync_state_mut(|state| *state = SyncState::Syncing);
                    return Command::perform(
                        client_sync_from_server(addr),
                        crate::Message::SyncResult,
                    );
                } else {
                    self.with_sync_state_mut(|state| *state = SyncState::IpAddrParseError);
                }
            }
            Message::ClearAddrInput => {
                self.with_ip_mut(|ip| ip.clear());
                self.with_sync_state_mut(|state| *state = SyncState::Waiting);

                self.with_port_mut(|port| *port = 2345);
            }
            Message::SyncToServer => {
                if let Ok(ip) = IpAddr::from_str(self.borrow_ip()) {
                    let addr = SocketAddr::new(ip, *self.borrow_port());
                    self.with_sync_state_mut(|state| *state = SyncState::Syncing);
                    return Command::perform(
                        client_sync_to_server(addr),
                        crate::Message::SyncResult,
                    );
                } else {
                    self.with_sync_state_mut(|state| *state = SyncState::IpAddrParseError);
                }
            }
            Message::IpAddrVerify => {
                if IpAddr::from_str(&self.borrow_ip()).is_err() {
                    self.with_sync_state_mut(|state| *state = SyncState::IpAddrParseError);
                } else {
                    self.with_sync_state_mut(|state| *state = SyncState::IpAddrParsePass);
                }
            }
            Message::SyncFromFile => {
                if let Some(path) = get_sync_file_path() {
                    self.with_sync_state_mut(|state| *state = SyncState::Syncing);

                    return Command::perform(sync_via_file(path, conn), crate::Message::SyncOption);
                } else {
                    self.with_sync_state_mut(|state| *state = SyncState::FilePathGetError);
                }
            }
            Message::OpenServer => {
                self.with_server_state_mut(|state| *state = ServerState::Starting);

                return Command::perform(
                    start_server(*self.borrow_port()),
                    crate::Message::StartServerResult,
                );
            }
            Message::Waiting => {
                // waiting...
                if *self.borrow_server_state() == ServerState::Error {
                    self.with_server_state_mut(|state| *state = ServerState::Closed);
                }
            }
            Message::CloseServer => {
                self.with_server_state_mut(|state| *state = ServerState::Closing);
                if let Some(cmd) = self.with_stop_mut(|stop| {
                    if let Some(stop) = stop.take() {
                        Some(Command::perform(
                            stop_server(stop),
                            crate::Message::ServerOption,
                        ))
                    } else {
                        None
                    }
                }) {
                    return cmd;
                } else {
                    self.with_server_state_mut(|state| *state = ServerState::Closed);
                }
            }
        }
        Command::none()
    }
}

impl Default for SyncView {
    fn default() -> Self {
        SyncViewBuilder {
            ip: String::new(),
            port: 2345,
            server_addr: String::new(),
            ip_qr_code: qr_code::Data::new(&[0]).unwrap(),
            sync_state: SyncState::Waiting,
            server_state: ServerState::Closed,
            stop: None,
            translate_builder: |server_addr: &String| {
                translate::TranslateWithArgs::new("ip-qr", translate::args("ip", server_addr))
            },
        }
        .build()
    }
}

pub static IP_REGEX_SET: OnceCell<RegexSet> = OnceCell::new();

pub async fn client_sync_from_server(addr: SocketAddr) -> anyhow::Result<()> {
    localnative_core::rpc::client::run_sync_from_server(&addr).await
}

pub async fn client_sync_to_server(addr: SocketAddr) -> anyhow::Result<()> {
    localnative_core::rpc::client::run_sync_to_server(&addr).await
}

pub fn get_sync_file_path() -> Option<PathBuf> {
    dirs::desktop_dir()
        .unwrap_or_else(std::env::temp_dir)
        .to_str()
        .and_then(|path| {
            open_file_dialog(
                &tr!("sync-file-title"),
                path,
                Some((&["*.sqlite3"], &tr!("sync-file"))),
            )
        })
        .map(PathBuf::from)
}

pub async fn sync_via_file(path: PathBuf, conn: Conn) -> Option<()> {
    tokio::task::spawn(async move {
        if let Some(uri) = path.to_str() {
            let conn = &*conn.lock().await;
            localnative_core::cmd::sync_via_attach(conn, uri);
        }
    })
    .await
    .map_err(error_handle)
    .ok()
}

pub fn get_ip() -> Option<String> {
    use std::net::UdpSocket;
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| s.connect("8.8.8.8:90").and_then(|_| s.local_addr()))
        .map(|addr| addr.ip().to_string())
        .map_err(error_handle)
        .ok()
}

pub async fn start_server(port: u16) -> std::io::Result<Stop> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
    localnative_core::rpc::server::iced_start_server(addr).await
}

pub async fn stop_server(stop: Stop) -> Option<()> {
    let res = stop.await.map_err(error_handle).ok()?;
    drop(res);
    Some(())
}
