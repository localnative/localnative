use iced::pure::widget::{qr_code, Column, QRCode};
use iced::pure::widget::{Button, Row, Text, TextInput};
use iced::pure::Element;
use iced::Command;
use iced_aw::pure::NumberInput;
use once_cell::sync::OnceCell;
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

use crate::{args, tr, Conn};

use crate::{
    error_handle,
    icons::IconItem,
    style::{self, Theme},
};

pub struct SyncView {
    ip: String,
    pub port: u16,
    pub server_addr: String,
    pub ip_qr_code: qr_code::State,
    pub sync_state: SyncState,
    pub server_state: ServerState,
    pub stop: Option<Stop>,
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
    pub fn new() -> Self {
        Self {
            ip: String::new(),
            port: 2345,
            server_state: ServerState::Closed,
            server_addr: String::new(),
            ip_qr_code: qr_code::State::new(&[0]).unwrap(),
            sync_state: SyncState::Waiting,
            stop: None,
        }
    }
    pub fn view(&self, theme: Theme) -> Element<Message> {
        let ip_input = TextInput::new("xxx.xxx.xxx.xxx", &self.ip, Message::IpInput)
            .padding(0)
            .on_submit(Message::IpAddrVerify);

        let ip_tip = iced::pure::widget::Tooltip::new(
            ip_input,
            r"输入格式:xxx.xxx.xxx.xxx",
            iced::tooltip::Position::Bottom,
        );

        let port_input = NumberInput::new(self.port, u16::MAX, Message::PortInput)
            .padding(0)
            .on_submit(Message::IpAddrVerify);

        let clear_button = Button::new(IconItem::Clear)
            .style(style::transparent(theme))
            .padding(0)
            .on_press(Message::ClearAddrInput);

        let ip_input_row = Row::new()
            .push(style::horizontal_rule())
            .push(Text::new(tr!("input-ip")))
            .push(ip_tip)
            .push(Text::new(":"))
            .push(port_input)
            .push(clear_button)
            .push(style::horizontal_rule());

        let sync_from_server_button = Button::new(
            Row::new()
                .push(IconItem::SyncFromServer)
                .push(Text::new(tr!("sync-from-server"))),
        )
        .style(style::transparent(theme))
        .padding(0)
        .on_press(Message::SyncFromServer);

        let sync_to_server_button = Button::new(
            Row::new()
                .push(IconItem::SyncToServer)
                .push(Text::new(tr!("sync-to-server"))),
        )
        .style(style::transparent(theme))
        .padding(0)
        .on_press(Message::SyncToServer);

        let sync_form_file_button = Button::new(
            Row::new()
                .push(IconItem::SyncFromFile)
                .push(Text::new(tr!("sync-from-file"))),
        )
        .style(style::transparent(theme))
        .padding(0)
        .on_press(Message::SyncFromFile);

        let text = match self.sync_state {
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

        let server_button_text = match self.server_state {
            ServerState::Closed => Row::new()
                .push(IconItem::CloseServer)
                .push(Text::new(tr!("closed"))),
            ServerState::Starting => Row::new()
                .push(IconItem::Sync)
                .push(Text::new(tr!("starting"))),
            ServerState::Opened => Row::new()
                .push(IconItem::OpenServer)
                .push(Text::new(tr!("opened"))),
            ServerState::Closing => Row::new()
                .push(IconItem::Sync)
                .push(Text::new(tr!("closing"))),
            ServerState::Error => Row::new()
                .push(IconItem::Clear)
                .push(Text::new(tr!("unknow-error"))),
        };
        let mut server_button = Button::new(server_button_text)
            .padding(0)
            .style(style::transparent(theme));

        server_button = match self.server_state {
            ServerState::Closed => server_button.on_press(Message::OpenServer),
            ServerState::Starting | ServerState::Closing | ServerState::Error => {
                server_button.on_press(Message::Waiting)
            }
            ServerState::Opened => server_button.on_press(Message::CloseServer),
        };

        let mut res = Column::new()
            .spacing(20)
            .push(Text::new(text))
            .push(Text::new(tr!("sync-client-tip")))
            .push(Text::new(tr!("input-ip-tip")))
            .push(ip_input_row)
            .push(
                Row::new()
                    .push(sync_from_server_button)
                    .push(sync_to_server_button)
                    .push(sync_form_file_button)
                    .spacing(20)
                    .align_items(iced::Alignment::Center),
            )
            .align_items(iced::Alignment::Center)
            .push(Text::new(tr!("sync-server-tip")))
            .push(server_button);
        if self.server_state == ServerState::Opened {
            println!("ip: {}", self.server_addr.clone());
            let args = args!("ip"=>self.server_addr.clone());
            res = res
                .push(Text::new(tr!("ip-qr";&args)))
                .push(QRCode::new(&self.ip_qr_code));
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
                    self.ip = input;
                }
            }
            Message::PortInput(input) => {
                self.port = input;
            }
            Message::SyncFromServer => {
                if let Ok(ip) = IpAddr::from_str(&self.ip) {
                    let addr = SocketAddr::new(ip, self.port);
                    self.sync_state = SyncState::Syncing;
                    return Command::perform(
                        client_sync_from_server(addr),
                        crate::Message::SyncResult,
                    );
                } else {
                    self.sync_state = SyncState::IpAddrParseError;
                }
            }
            Message::ClearAddrInput => {
                self.ip.clear();
                self.sync_state = SyncState::Waiting;
                self.port = 2345;
            }
            Message::SyncToServer => {
                if let Ok(ip) = IpAddr::from_str(&self.ip) {
                    let addr = SocketAddr::new(ip, self.port);
                    self.sync_state = SyncState::Syncing;
                    return Command::perform(
                        client_sync_to_server(addr),
                        crate::Message::SyncResult,
                    );
                } else {
                    self.sync_state = SyncState::IpAddrParseError;
                }
            }
            Message::IpAddrVerify => {
                if IpAddr::from_str(&self.ip).is_err() {
                    self.sync_state = SyncState::IpAddrParseError;
                } else {
                    self.sync_state = SyncState::IpAddrParsePass;
                }
            }
            Message::SyncFromFile => {
                if let Some(path) = get_sync_file_path() {
                    self.sync_state = SyncState::Syncing;
                    return Command::perform(sync_via_file(path, conn), crate::Message::SyncOption);
                } else {
                    self.sync_state = SyncState::FilePathGetError;
                }
            }
            Message::OpenServer => {
                self.server_state = ServerState::Starting;
                return Command::perform(
                    start_server(self.port),
                    crate::Message::StartServerResult,
                );
            }
            Message::Waiting => {
                // waiting...
                if self.server_state == ServerState::Error {
                    self.server_state = ServerState::Closed;
                }
            }
            Message::CloseServer => {
                self.server_state = ServerState::Closing;
                if let Some(stop) = self.stop.take() {
                    return Command::perform(stop_server(stop), crate::Message::ServerOption);
                } else {
                    self.server_state = ServerState::Closed;
                }
            }
        }
        Command::none()
    }
}

impl Default for SyncView {
    fn default() -> Self {
        Self::new()
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
    localnative_core::dirs::desktop_dir()
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
