use std::{fmt::Display, net::SocketAddr, str::FromStr};

use directories_next::BaseDirs;
use iced::{
    button, pick_list, qr_code, slider, text_input, Button, Column, Element, PickList, Row, Rule,
    Slider, Text, TextInput,
};
use serde::{Deserialize, Serialize};

use crate::style::{
    symbol::{self, Symbol},
    Theme,
};

#[derive(Debug, Clone)]
pub enum Message {
    LimitChanged(u32),
    LanguageChanged(Language),
    BackendChanged(Backend),
    AddrsChanged(String),
    SelectSettingBoard,
    Server,
    OpenFile,
    Sync,
    ThemeChanged,
    OpenSlider,
    HideSlider,
    Apply,
    Reset,
    ClearAddrInput,
    FixHost,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Config {
    pub limit: u32,
    pub language: Language,
    pub theme: Theme,
    pub is_created_db: bool,
}
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Gl,
    Vulkan,
    #[cfg(target_os = "windows")]
    Dx12,
    #[cfg(target_os = "windows")]
    Dx11,
    #[cfg(target_os = "macos")]
    Metal,
}
impl Backend {
    pub fn from_env() -> Backend {
        let backend = std::env::var(crate::BACKEND);
        log::debug!("backend from env:{:?}", &backend);
        if let Ok(backend) = backend {
            match backend.to_ascii_lowercase().as_str() {
                "opengl" => Backend::Gl,
                "vulkan" => Backend::Vulkan,
                #[cfg(target_os = "windows")]
                "dx12" => Backend::Dx12,
                #[cfg(target_os = "windows")]
                "dx11" => Backend::Dx11,
                #[cfg(target_os = "macos")]
                "metal" => Backend::Metal,
                _ => Backend::default(),
            }
        } else {
            Backend::default()
        }
    }
    pub async fn from_file() -> anyhow::Result<Backend> {
        let file_path = app_dir().join(".env");
        let backend = String::from_utf8(tokio::fs::read(file_path).await?)?;
        log::debug!("backend: {}", &backend[15..]);
        match backend[15..].to_ascii_lowercase().as_str() {
            "opengl" => Ok(Backend::Gl),
            "vulkan" => Ok(Backend::Vulkan),
            #[cfg(target_os = "windows")]
            "dx12" => Ok(Backend::Dx12),
            #[cfg(target_os = "windows")]
            "dx11" => Ok(Backend::Dx11),
            #[cfg(target_os = "macos")]
            "metal" => Ok(Backend::Metal),
            s => Err(anyhow::anyhow!("Unknow Backend:{:?}", s)),
        }
    }
}
impl Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Backend::Gl => f.write_str("OpenGL"),
            Backend::Vulkan => f.write_str("Vulkan"),
            #[cfg(target_os = "windows")]
            Backend::Dx12 => f.write_str("Dx12"),
            #[cfg(target_os = "windows")]
            Backend::Dx11 => f.write_str("Dx11"),
            #[cfg(target_os = "macos")]
            Backend::Metal => f.write_str("Metal"),
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        let res = Self::Dx11;
        #[cfg(target_os = "macos")]
        let res = Self::Metal;
        #[cfg(target_os = "linux")]
        let res = Self::Vulkan;
        res
    }
}

impl Config {
    pub async fn new() -> Self {
        Config::default()
    }
}
#[derive(Debug, Default)]
pub struct SettingView {
    pub config: Config,
    pub state: State,
    pub board_state: BoardState,
    pub sync_state: SyncState,
}
#[derive(Debug, Clone, Default)]
pub struct State {
    setting_button: button::State,
    theme_button: button::State,
    server_button: button::State,
    open_file_button: button::State,
    clear_button: button::State,
    addr_input: text_input::State,
    addr: String,
    pub socket: Option<SocketAddr>,
    offset: slider::State,
}

#[derive(Debug, Clone, Default)]
pub struct BoardState {
    limit_temp: u32,
    is_open: bool,
    limit_button: button::State,
    limit_slider: slider::State,
    apply_button: button::State,
    reset_button: button::State,
    fix_web_host_button: button::State,
    language: pick_list::State<Language>,
    language_temp: Language,
    backend: pick_list::State<Backend>,
    pub backend_temp: Backend,
    pub backend_org: Backend,
}

#[derive(Debug)]
pub struct SyncState {
    qr_code: qr_code::State,
    pub qr_data: String,
}

impl Default for SyncState {
    fn default() -> Self {
        let qrcode = if let Ok(qr_state) = qr_code::State::with_version(
            "",
            qr_code::Version::Normal(8),
            qr_code::ErrorCorrection::Low,
        ) {
            qr_state
        } else {
            qr_code::State::new("").unwrap_or_else(
                // 如果到了这里，都出错，只能panic了
                |e| qr_code::State::new(format!("error in qrcode generation:{:?}", e)).unwrap(),
            )
        };
        Self {
            qr_code: qrcode,
            qr_data: String::default(),
        }
    }
}
impl SettingView {
    pub fn init(config: Config, ip: Option<String>) -> Self {
        let qr_data;
        let qr_code = if let Some(ip) = ip {
            qr_data = ip;
            qr_code::State::new(&qr_data).unwrap_or_else(|e| {
                qr_code::State::new(format!("Error in qrcode generation: {:?}", e)).unwrap()
            })
        } else {
            qr_data = "0.0.0.0:2345".to_owned();
            let data = "抱歉，获取本地ip失败。";
            qr_code::State::new(&data).unwrap_or_else(|e| {
                qr_code::State::new(format!("Error in qrcode generation: {:?}", e)).unwrap()
            })
        };
        let backend = Backend::from_env();
        Self {
            config,
            board_state: BoardState {
                limit_temp: config.limit,
                language_temp: config.language,
                is_open: false,
                backend_temp: backend,
                backend_org: backend,
                ..Default::default()
            },
            sync_state: SyncState { qr_code, qr_data },
            ..Default::default()
        }
    }
    pub fn limit(&self) -> u32 {
        self.config.limit
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::LimitChanged(limit) => {
                self.board_state.limit_temp = limit;
            }
            Message::LanguageChanged(language) => {
                self.board_state.language_temp = language;
            }
            Message::ThemeChanged => {
                let old = self.config.theme;
                self.config.theme = -old;
            }
            Message::OpenSlider => {
                self.board_state.is_open = true;
            }
            Message::HideSlider => {
                self.board_state.is_open = false;
            }
            Message::Apply => {
                self.config.limit = self.board_state.limit_temp;
                self.config.language = self.board_state.language_temp;
            }
            Message::SelectSettingBoard => {}
            Message::Server => {}
            Message::OpenFile => {}
            Message::Sync => {
                let addr = SocketAddr::from_str(&self.state.addr);
                match addr {
                    Ok(socket) => self.state.socket.replace(socket),
                    Err(_) => self.state.socket.take(),
                };
            }
            Message::BackendChanged(backend) => {
                self.board_state.backend_temp = backend;
            }
            Message::Reset => {
                if self.board_state.backend_temp != Backend::from_env() {
                    self.board_state.backend_temp = Backend::from_env();
                }
                if self.board_state.limit_temp != self.config.limit {
                    self.board_state.limit_temp = self.config.limit;
                }
                if self.board_state.language_temp != self.config.language {
                    self.board_state.language_temp = self.config.language;
                }
            }
            Message::AddrsChanged(addr) => {
                self.state.addr = addr;
            }
            Message::ClearAddrInput => {
                self.state.addr.clear();
                if self.state.socket.is_some() {
                    self.state.socket.take();
                }
            }
            Message::FixHost => {}
        }
    }
    pub fn viwe(&mut self) -> Element<Message> {
        let SettingView { config, state, .. } = self;
        left_bar_viwe(state, config.theme)
    }
    pub fn setting_board_open_view(&mut self) -> Element<Message> {
        let SettingView {
            config,
            state,
            board_state,
            ..
        } = self;
        let left_bar = left_bar_viwe(state, config.theme);
        let setting_board = setting_board_view(board_state, &*config);
        Row::new().push(left_bar).push(setting_board).into()
    }
    pub fn sync_board_open_view(&mut self) -> Element<Message> {
        let SettingView {
            config,
            state,
            sync_state,
            ..
        } = self;
        let left_bar = left_bar_viwe(state, config.theme);
        let sync_board = sync_board_view(&*sync_state);
        Row::new().push(left_bar).push(sync_board).into()
    }
}
fn sync_board_view(state: &SyncState) -> Element<Message> {
    let SyncState { qr_code, qr_data } = state;
    Column::new()
        .height(iced::Length::Fill)
        .width(iced::Length::FillPortion(10))
        .align_items(iced::Align::Center)
        .push(Text::new(
            "Click Stop Server button in main window to stop server.",
        ))
        .push(Text::new(format!(
            "Use {} in Local Native desktop app for server address and port to start sync.",
            qr_data
        )))
        .push(Text::new(
            "Use Local Native mobile app to scan this barcode to start sync.",
        ))
        .push(iced::QRCode::new(qr_code).cell_size(10))
        .into()
}
fn left_bar_viwe(state: &mut State, theme: Theme) -> Element<Message> {
    let State {
        theme_button,
        setting_button,
        server_button,
        open_file_button,
        clear_button,
        addr_input,
        addr,
        ..
    } = state;

    let theme_button = Button::new(theme_button, {
        if theme == Theme::Dark {
            crate::style::icon::Icon::dark()
        } else {
            crate::style::icon::Icon::light()
        }
    })
    .style(Symbol)
    .on_press(Message::ThemeChanged);

    let server_button = Button::new(server_button, Text::new("server")).on_press(Message::Server);
    let clear_button = Button::new(
        clear_button,
        crate::style::icon::Icon::close()
            .width(iced::Length::Units(18))
            .height(iced::Length::Units(18)),
    )
    .style(symbol::Symbol)
    .on_press(Message::ClearAddrInput);
    let addr_input = TextInput::new(
        addr_input,
        "xxx.xxx.xxx.xxx:2345",
        &addr,
        Message::AddrsChanged,
    )
    .on_submit(Message::Sync);
    let addr_row = Column::new()
        .push(Text::new("start client sync"))
        .push(Row::new().push(addr_input).push(clear_button))
        .push(Text::new("input format:\n [server address]:[port]"));
    let open_file_button = Button::new(open_file_button, Text::new("sync via attach file"))
        .on_press(Message::OpenFile);
    let setting_button = Button::new(
        setting_button,
        Row::new()
            .push(crate::style::icon::Icon::settings())
            .push(Text::new("setting")),
    )
    .style(Symbol)
    .on_press(Message::SelectSettingBoard);
    Column::new()
        .align_items(iced::Align::Center)
        .height(iced::Length::Fill)
        .width(iced::Length::FillPortion(2))
        .spacing(10)
        .push(open_file_button)
        .push(server_button)
        .push(addr_row)
        .push(Rule::vertical(50).style(symbol::Symbol))
        .push(theme_button)
        .push(setting_button)
        .into()
}
fn setting_board_view<'a>(
    board_state: &'a mut BoardState,
    config: &'a Config,
) -> Element<'a, Message> {
    let BoardState {
        limit_button,
        limit_slider,
        apply_button,
        language,
        backend,
        reset_button,
        fix_web_host_button,
        ..
    } = board_state;
    let limit_text = Text::new(format!("limit: {}", board_state.limit_temp));
    let setting_column = if board_state.is_open {
        let limit_ctrl = Slider::new(
            limit_slider,
            5..=50,
            board_state.limit_temp,
            Message::LimitChanged,
        )
        .on_release(Message::HideSlider);
        Column::new().push(limit_text).push(limit_ctrl)
    } else {
        Column::new().push(
            Button::new(limit_button, limit_text)
                .style(Symbol)
                .on_press(Message::OpenSlider),
        )
    };
    let language = Row::new()
        .spacing(300)
        .push(Text::new("language"))
        .push(PickList::new(
            language,
            &[Language::Chinese, Language::English][..],
            Some(board_state.language_temp),
            Message::LanguageChanged,
        ));
    let backends = {
        #[cfg(target_os = "windows")]
        let res = &[Backend::Gl, Backend::Vulkan, Backend::Dx11, Backend::Dx12][..];
        #[cfg(target_os = "macos")]
        let res = &[Backend::Gl, Backend::Vulkan, Backend::Metal][..];
        #[cfg(target_os = "linux")]
        let res = &[Backend::Gl, Backend::Vulkan][..];
        res
    };
    let backend = Row::new()
        .spacing(300)
        .push(Text::new("render backend"))
        .push(PickList::new(
            backend,
            backends,
            Some(board_state.backend_temp),
            Message::BackendChanged,
        ));

    let apply = if config.language != board_state.language_temp
        || config.limit != board_state.limit_temp
        || board_state.backend_org != board_state.backend_temp
    {
        Some(
            Row::new()
                .push(Rule::horizontal(50).style(symbol::Symbol))
                .spacing(2)
                .push(
                    Button::new(
                        reset_button,
                        Row::new()
                            .align_items(iced::Align::Center)
                            .push(crate::style::icon::Icon::reset())
                            .push(Text::new("reset")),
                    )
                    .style(symbol::Symbol)
                    .on_press(Message::Reset),
                )
                .push(
                    Button::new(
                        apply_button,
                        Row::new()
                            .align_items(iced::Align::Center)
                            .push(crate::style::icon::Icon::enter())
                            .push(Text::new("apply setting")),
                    )
                    .style(symbol::Symbol)
                    .on_press(Message::Apply),
                ),
        )
    } else {
        None
    };
    let fix_button = Button::new(fix_web_host_button, Text::new("Try fix your web ext host"))
        .on_press(Message::FixHost);
    let res = setting_column
        .align_items(iced::Align::Center)
        .height(iced::Length::Fill)
        .width(iced::Length::FillPortion(10))
        .push(language)
        .push(backend)
        .push(fix_button)
        .push(Rule::vertical(50).style(symbol::Symbol));
    if let Some(apply) = apply {
        res.push(apply)
    } else {
        res
    }
    .into()
}
#[derive(Debug, Clone)]
pub enum LoadError {
    FileError,
    FormatError,
}
#[derive(Debug, Clone)]
pub enum ConfigError {
    Load(LoadError),
    Save(SaveError),
}
#[derive(Debug, Clone)]
pub enum SaveError {
    FileError,
    WriteError,
    FormatError,
}

impl Config {
    pub fn path() -> std::path::PathBuf {
        let mut path = app_dir().join("config");
        path.push("config.json");
        path
    }
    pub async fn load() -> Result<Self, ConfigError> {
        use tokio::io::AsyncReadExt;

        let mut contents = String::new();

        let mut file = tokio::fs::File::open(Self::path())
            .await
            .map_err(|_| ConfigError::Load(LoadError::FileError))?;
        file.read_to_string(&mut contents)
            .await
            .map_err(|_| ConfigError::Load(LoadError::FileError))?;

        serde_json::from_str(&contents).map_err(|_| ConfigError::Load(LoadError::FormatError))
    }
    pub async fn save(self) -> Result<Config, ConfigError> {
        use tokio::io::AsyncWriteExt;

        let json = serde_json::to_string_pretty(&self)
            .map_err(|_| ConfigError::Save(SaveError::FormatError))?;

        let path = Self::path();

        if let Some(dir) = path.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|_| ConfigError::Save(SaveError::FileError))?;
        }

        {
            let mut file = tokio::fs::File::create(path)
                .await
                .map_err(|_| ConfigError::Save(SaveError::FileError))?;

            file.write_all(json.as_bytes())
                .await
                .map_err(|_| ConfigError::Save(SaveError::WriteError))?;
        }

        Ok(self)
    }
}
pub fn app_dir() -> std::path::PathBuf {
    if let Some(base) = BaseDirs::new() {
        base.home_dir().join("LocalNative")
    } else {
        log::error!("init app dir fial.");
        std::env::current_dir().unwrap().join("LocalNative")
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}
impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}
impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::English => write!(f, "english"),
            Language::Chinese => write!(f, "中文"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            limit: 10,
            theme: Theme::Light,
            language: Language::English,
            is_created_db: false,
        }
    }
}
