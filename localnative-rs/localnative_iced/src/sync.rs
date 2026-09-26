use iced::Element;
use iced::Task;
use iced::widget::{QRCode, Space, button, column, qr_code, row, text, text_input};
use iced_aw::NumberInput;

use localnative_core::db::Pool;
use localnative_core::discovery::PeerInfo;
use std::net::IpAddr;
use std::str::FromStr;
use std::{net::SocketAddr, path::PathBuf};

use tinyfiledialogs::open_file_dialog;

use crate::tr;

use crate::icons::IconItem;

/// The address the sync server binds to.
const SERVER_BIND: &str = "0.0.0.0";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DiscoveryState {
    #[default]
    Idle,
    Scanning,
    Done,
    Error(String),
}

#[derive(Debug, Default)]
pub struct SyncView {
    pub ip: String,
    pub port: u16,
    /// Pairing code typed for first contact with a new peer.
    pub code: String,
    pub server_addr: String,
    /// Shown while this device accepts a new pairing.
    pub pairing_code: Option<String>,
    pub ip_qr_code: Option<qr_code::Data>,
    pub sync_state: SyncState,
    pub server_state: ServerState,
    pub discovered_peers: Vec<PeerInfo>,
    pub discovery_state: DiscoveryState,
}

impl SyncView {
    pub fn new() -> Self {
        Self {
            port: 2345,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SyncState {
    #[default]
    Waiting,
    Syncing,
    SyncError(String),
    Complete,
    IpAddrParseError,
    IpAddrParsePass,
    FilePathGetError,
    SyncFromFileError(String),
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum ServerState {
    #[default]
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
    CodeInput(String),
    ClearAddrInput,
    /// Sync with the peer at ip:port; the pairing code is used when set.
    SyncPeer,
    SyncFromFile,
    IpAddrVerify,
    OpenServer,
    /// Accept one new device: shows a pairing code.
    StartPairing,
    Waiting,
    CloseServer,
    DiscoverPeers,
    SelectPeer(usize),
}

impl SyncView {
    pub fn view(&self) -> Element<'_, Message> {
        let ip_input = text_input("xxx.xxx.xxx.xxx", &self.ip)
            .on_input(Message::IpInput)
            .on_submit(Message::IpAddrVerify);

        let port_input = NumberInput::new(&self.port, 0..=u16::MAX, Message::PortInput).padding(0.);

        let code_input =
            text_input(&tr!("pairing-code-placeholder"), &self.code).on_input(Message::CodeInput);

        let clear_button = button(IconItem::Clear)
            .padding(0)
            .on_press(Message::ClearAddrInput);

        let ip_input_row = row![
            Space::new().width(iced::Length::Fill),
            text(tr!("input-ip")),
            ip_input,
            text(":"),
            port_input,
            clear_button,
            Space::new().width(iced::Length::Fill),
        ];

        // Pairing code: only needed the first time two devices sync.
        let code_row = row![
            Space::new().width(iced::Length::Fill),
            text(tr!("pairing-code")),
            code_input,
            Space::new().width(iced::Length::Fill),
        ];

        // --- Peer discovery section ---
        let discover_button_label = match self.discovery_state {
            DiscoveryState::Idle | DiscoveryState::Done | DiscoveryState::Error(_) => {
                text(tr!("discover-peers"))
            }
            DiscoveryState::Scanning => text(tr!("discover-peers-scanning")),
        };

        let mut discover_button = button(row![IconItem::Sync, discover_button_label]).padding(0);
        if self.discovery_state != DiscoveryState::Scanning {
            discover_button = discover_button.on_press(Message::DiscoverPeers);
        }

        let mut discovery_col = column![discover_button].spacing(4);

        if let DiscoveryState::Error(err) = &self.discovery_state {
            discovery_col = discovery_col.push(text(err.to_string()));
        }

        let peers = &self.discovered_peers;
        if !peers.is_empty() {
            for (idx, peer) in peers.iter().enumerate() {
                let ip_str = peer
                    .addresses
                    .first()
                    .map(|a| a.to_string())
                    .unwrap_or_default();
                let label = format!(
                    "{} — {ip_str}:{} (proto {})",
                    peer.hostname, peer.port, peer.protocol
                );
                let peer_button = button(text(label))
                    .padding(2)
                    .on_press(Message::SelectPeer(idx));
                discovery_col = discovery_col.push(peer_button);
            }
        } else if self.discovery_state == DiscoveryState::Done {
            discovery_col = discovery_col.push(text(tr!("no-peers-found")));
        }

        let sync_with_peer_button =
            button(row![IconItem::SyncFromServer, text(tr!("sync-with-peer"))])
                .padding(0)
                .on_press(Message::SyncPeer);

        let sync_from_file_button =
            button(row![IconItem::SyncFromFile, text(tr!("sync-from-file"))])
                .padding(0)
                .on_press(Message::SyncFromFile);

        let content_text: String = match &self.sync_state {
            SyncState::Waiting => tr!("sync-waiting").into_owned(),
            SyncState::Syncing => tr!("sync-syncing").into_owned(),
            SyncState::SyncError(err) => format!("{}{err}", tr!("sync-error")),
            SyncState::Complete => tr!("sync-complete").into_owned(),
            SyncState::IpAddrParseError => tr!("sync-ip-parse-error").into_owned(),
            SyncState::IpAddrParsePass => tr!("sync-ip-parse-complete").into_owned(),
            SyncState::FilePathGetError => tr!("sync-file-path-error").into_owned(),
            SyncState::SyncFromFileError(err) => {
                format!("{}{err}", tr!("sync-error"))
            }
        };

        let server_button_text = match self.server_state {
            ServerState::Closed => row![IconItem::CloseServer, text(tr!("closed"))],
            ServerState::Starting => row![IconItem::Sync, text(tr!("starting"))],
            ServerState::Opened => row![IconItem::OpenServer, text(tr!("opened"))],
            ServerState::Closing => row![IconItem::Sync, text(tr!("closing"))],
            ServerState::Error => row![IconItem::Clear, text(tr!("unknow-error"))],
        };
        let mut server_button = button(server_button_text).padding(0);

        server_button = match self.server_state {
            ServerState::Closed => server_button.on_press(Message::OpenServer),
            ServerState::Starting | ServerState::Closing | ServerState::Error => {
                server_button.on_press(Message::Waiting)
            }
            ServerState::Opened => server_button.on_press(Message::CloseServer),
        };

        let mut server_section = column![text(tr!("sync-server-tip")), server_button].spacing(8);
        if let Some(code) = &self.pairing_code {
            // The code another device types to pair with this one.
            server_section = server_section.push(
                row![
                    text(tr!("pairing-code-showing")),
                    text(code.clone()).size(24)
                ]
                .spacing(8),
            );
        }

        let mut res = column![
            text(content_text),
            text(tr!("sync-client-tip")),
            text(tr!("input-ip-tip")),
            ip_input_row,
            code_row,
            discovery_col,
            row![sync_with_peer_button, sync_from_file_button].spacing(20),
            server_section,
        ]
        .spacing(20)
        .align_x(iced::Alignment::Center);

        if self.server_state == ServerState::Opened
            && let Some(qr) = &self.ip_qr_code
        {
            res = res
                .push(text(self.server_addr.clone()))
                .push(QRCode::new(qr));
        }

        res.into()
    }

    pub fn update(&mut self, message: Message, pool: Pool) -> Task<crate::Message> {
        match message {
            Message::IpInput(input) => self.ip = input,
            Message::PortInput(input) => self.port = input,
            Message::CodeInput(input) => self.code = input,
            Message::SyncPeer => {
                match SocketAddr::from_str(&format!("{}:{}", self.ip.trim(), self.port)) {
                    Ok(addr) => {
                        self.sync_state = SyncState::Syncing;
                        let code = self.code.trim().to_string();
                        let code = (!code.is_empty()).then_some(code);
                        return Task::perform(
                            client_sync(addr, code, pool.clone()),
                            crate::Message::SyncResult,
                        );
                    }
                    Err(_) => self.sync_state = SyncState::IpAddrParseError,
                }
            }
            Message::ClearAddrInput => {
                self.ip.clear();
                self.code.clear();
                self.sync_state = SyncState::Waiting;
                self.port = 2345;
            }
            Message::IpAddrVerify => {
                self.sync_state = if IpAddr::from_str(self.ip.trim()).is_err() {
                    SyncState::IpAddrParseError
                } else {
                    SyncState::IpAddrParsePass
                };
            }
            Message::SyncFromFile => {
                if let Some(path) = get_sync_file_path() {
                    self.sync_state = SyncState::Syncing;
                    return Task::perform(
                        sync_via_file(path, pool.clone()),
                        crate::Message::SyncFileResult,
                    );
                } else {
                    self.sync_state = SyncState::FilePathGetError;
                }
            }
            Message::OpenServer => {
                self.server_state = ServerState::Starting;
                let addr = format!("{SERVER_BIND}:{}", self.port);
                return Task::perform(
                    start_server(addr, pool.clone()),
                    crate::Message::ServerStartResult,
                );
            }
            Message::StartPairing => {
                self.server_state = ServerState::Starting;
                let addr = format!("{SERVER_BIND}:{}", self.port);
                return Task::perform(
                    start_pairing(addr, pool.clone()),
                    crate::Message::ServerStartResult,
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
                let addr = format!("{SERVER_BIND}:{}", self.port);
                return Task::perform(stop_server(addr), crate::Message::ServerStopResult);
            }
            Message::DiscoverPeers => {
                self.discovery_state = DiscoveryState::Scanning;
                self.discovered_peers.clear();
                return Task::perform(discover_lan_peers(), crate::Message::DiscoveryResult);
            }
            Message::SelectPeer(idx) => {
                if let Some(peer) = self.discovered_peers.get(idx)
                    && let Some(addr) = peer.addresses.first()
                {
                    self.ip = addr.to_string();
                    self.port = peer.port;
                    self.sync_state = SyncState::IpAddrParsePass;
                }
            }
        }
        Task::none()
    }
}

/// Sync with a peer through the JSON dispatcher on a worker thread, so no
/// database connection is ever held across the UI runtime's awaits.
pub async fn client_sync(
    addr: SocketAddr,
    code: Option<String>,
    pool: Pool,
) -> Result<String, String> {
    let _ = pool; // the dispatcher opens its own pool at the configured path
    let mut command = serde_json::json!({ "action": "client-sync", "addr": addr.to_string() });
    if let Some(code) = code {
        command["code"] = serde_json::json!(code);
    }
    let response =
        tokio::task::spawn_blocking(move || localnative_core::run_sync(&command.to_string()))
            .await
            .map_err(|e| e.to_string())?;
    match serde_json::from_str::<serde_json::Value>(&response) {
        Ok(value) => {
            if let Some(error) = value.get("error").and_then(|e| e.as_str()) {
                Err(error.to_string())
            } else {
                Ok(value
                    .get("client-sync")
                    .and_then(|s| s.as_str())
                    .unwrap_or("sync ok")
                    .to_string())
            }
        }
        Err(_) => Ok(response),
    }
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

pub async fn sync_via_file(path: PathBuf, pool: Pool) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let Some(uri) = path.to_str() else {
            return Err("selected path is not valid UTF-8".to_string());
        };
        let conn = pool
            .get()
            .map_err(|e| format!("database connection error: {e}"))?;
        localnative_core::db::queries::sync_via_attach(&conn, uri).map_err(|e| format!("{e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Start the sync server through the core's registry. Returns the address a
/// peer can use, and the pairing code when one was requested.
pub async fn start_server(addr: String, pool: Pool) -> Result<ServerStarted, String> {
    let command = serde_json::json!({ "action": "server", "addr": addr });
    run_server_command(command, pool).await
}

pub async fn start_pairing(addr: String, pool: Pool) -> Result<ServerStarted, String> {
    let command = serde_json::json!({ "action": "server-pairing", "addr": addr });
    run_server_command(command, pool).await
}

#[derive(Debug, Clone)]
pub struct ServerStarted {
    pub address: Option<String>,
    pub pairing_code: Option<String>,
}

async fn run_server_command(
    command: serde_json::Value,
    _pool: Pool,
) -> Result<ServerStarted, String> {
    let response =
        tokio::task::spawn_blocking(move || localnative_core::run_sync(&command.to_string()))
            .await
            .map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&response).map_err(|_| response.clone())?;
    if let Some(error) = value.get("error").and_then(|e| e.as_str()) {
        return Err(error.to_string());
    }
    Ok(ServerStarted {
        address: value
            .get("addresses")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|a| a.as_str())
            .map(str::to_owned),
        pairing_code: value
            .get("pairing-code")
            .and_then(|c| c.as_str())
            .map(str::to_owned),
    })
}

pub async fn stop_server(addr: String) -> Result<(), String> {
    let command = serde_json::json!({ "action": "server-stop", "addr": addr });
    let response =
        tokio::task::spawn_blocking(move || localnative_core::run_sync(&command.to_string()))
            .await
            .map_err(|e| e.to_string())?;
    match serde_json::from_str::<serde_json::Value>(&response) {
        Ok(value) if value.get("error").is_none() => Ok(()),
        Ok(value) => Err(value
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("stop failed")
            .to_string()),
        Err(_) => Err(response),
    }
}

/// Scan the LAN for Local Native peers via mDNS (3-second timeout).
pub async fn discover_lan_peers() -> Result<Vec<PeerInfo>, String> {
    localnative_core::discovery::discover_peers(std::time::Duration::from_secs(3))
        .await
        .map_err(|e| e.to_string())
}
