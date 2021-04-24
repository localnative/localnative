use std::fmt::Display;

use directories_next::BaseDirs;
use iced::{
    button, pick_list, qr_code, slider, Button, Column, Element, PickList, Row, Slider, Text,
};
use serde::{Deserialize, Serialize};

use crate::style::{symbol::Symbol, Theme};

#[derive(Debug, Clone)]
pub enum Message {
    LimitChanged(u32),
    LanguageChanged(Language),
    BackendChanged(Backend),
    SelectSettingBoard,
    SelectServer,
    OpenFile,
    OpenClient,
    ThemeChanged,
    OpenSlider,
    HideSlider,
    Apply,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Config {
    pub limit: u32,
    pub language: Language,
    pub theme: Theme,
    pub backend: Backend,
    pub is_created_db: bool,
}
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Gl,
    Vulkan,
    Dx12,
    Dx11,
    Metal,
}
impl Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Backend::Gl => f.write_str("OpenGL"),
            Backend::Vulkan => f.write_str("Vulkan"),
            Backend::Dx12 => f.write_str("Dx12"),
            Backend::Dx11 => f.write_str("Dx11"),
            Backend::Metal => f.write_str("Metal"),
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        if cfg!(target_os = "windows") {
            Self::Dx11
        } else if cfg!(target_os = "macos") {
            Self::Metal
        } else {
            Self::Vulkan
        }
    }
}

impl Config {
    pub async fn new() -> Self {
        Config::default()
    }
}
#[derive(Debug, Default)]
pub struct ConfigView {
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
    client_button: button::State,
    offset: slider::State,
}

#[derive(Debug, Clone, Default)]
pub struct BoardState {
    limit_temp: u32,
    is_open: bool,
    limit_button: button::State,
    limit_slider: slider::State,
    apply_button: button::State,
    language: pick_list::State<Language>,
    language_temp: Language,
    backend: pick_list::State<Backend>,
    backend_temp: Backend,
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
            qr_code::State::new("").unwrap_or(
                // 如果到了这里，都出错，只能panic了
                qr_code::State::new("Error in qrcode generation.").unwrap(),
            )
        };
        Self {
            qr_code: qrcode,
            qr_data: String::default(),
        }
    }
}
impl ConfigView {
    pub fn init(config: Config, ip: Option<String>) -> Self {
        let qr_data;
        let qr_code = if let Some(ip) = ip {
            qr_data = ip;
            qr_code::State::new(&qr_data)
                .unwrap_or(qr_code::State::new("Error in qrcode generation.").unwrap())
        } else {
            qr_data = "0.0.0.0:2345".to_owned();
            let data = "抱歉，获取本地ip失败。";
            qr_code::State::new(&data)
                .unwrap_or(qr_code::State::new("Error in qrcode generation.").unwrap())
        };
        Self {
            config,
            board_state: BoardState {
                limit_temp: config.limit,
                language_temp: config.language,
                is_open: false,
                backend_temp: config.backend,
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
                if self.config.backend != self.board_state.backend_temp {
                    self.config.backend = self.board_state.backend_temp;
                    if cfg!(target_os = "windows") {
                        use winreg::{enums::*, RegKey};
                        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                        let (env, _) = hkcu.create_subkey("Environment").unwrap(); // create_subkey opens with write permissions
                        env.set_value(crate::BACKEND, &self.board_state.backend_temp.to_string())
                            .unwrap();
                        log::info!("backend {:?}", std::env::var(crate::BACKEND));
                    } else {
                        //TODO:
                    }
                }
            }
            Message::SelectSettingBoard => {}
            Message::SelectServer => {}
            Message::OpenFile => {}
            Message::OpenClient => {}
            Message::BackendChanged(backend) => {
                self.board_state.backend_temp = backend;
            }
        }
    }
    pub fn viwe(&mut self) -> Element<Message> {
        let ConfigView { config, state, .. } = self;
        left_bar_viwe(state, config.theme, "start server".to_owned())
    }
    pub fn setting_board_open_view(&mut self) -> Element<Message> {
        let ConfigView {
            config,
            state,
            board_state,
            ..
        } = self;
        let left_bar = left_bar_viwe(state, config.theme, "start server".to_owned());
        let setting_board = setting_board_view(board_state);
        Row::new().push(left_bar).push(setting_board).into()
    }
    pub fn sync_board_open_view(&mut self) -> Element<Message> {
        let ConfigView {
            config,
            state,
            sync_state,
            ..
        } = self;
        let left_bar = left_bar_viwe(state, config.theme, "stop server".to_owned());
        let sync_board = sync_board_view(&*sync_state);
        Row::new().push(left_bar).push(sync_board).into()
    }
}
fn sync_board_view(state: &SyncState) -> Element<Message> {
    let SyncState { qr_code, qr_data } = state;
    let text = Column::new()
        .push(Text::new(
            "Click Stop Server button in main window to stop server.",
        ))
        .push(Text::new(format!(
            "Use {} in Local Native desktop app for server address and port to start sync.",
            qr_data
        )))
        .push(Text::new(
            "Use Local Native mobile app to scan this barcode to start sync.",
        ));
    crate::Wrap::new()
        .height(iced::Length::Fill)
        .width(iced::Length::FillPortion(10))
        .push(text.into())
        .push(iced::QRCode::new(qr_code).into())
        .into()
}
fn left_bar_viwe(state: &mut State, theme: Theme, server_text: String) -> Element<Message> {
    let State {
        theme_button,
        setting_button,
        server_button,
        open_file_button,
        client_button,
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
    let theme = Column::new().push(
        Row::new()
            .push(theme_button)
            .align_items(iced::Align::Center),
    );

    let server_button =
        Button::new(server_button, Text::new(server_text)).on_press(Message::SelectServer);
    let client_button =
        Button::new(client_button, Text::new("start client sync")).on_press(Message::OpenClient);
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
        .push(client_button)
        .push(setting_button)
        .push(theme)
        .into()
}
fn setting_board_view(board_state: &mut BoardState) -> Element<Message> {
    let BoardState {
        limit_button,
        limit_slider,
        apply_button,
        language,
        backend,
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
    let language = PickList::new(
        language,
        &[Language::Chinese, Language::English][..],
        Some(board_state.language_temp),
        Message::LanguageChanged,
    );
    let backends = if cfg!(target_os = "windows") {
        &[Backend::Gl, Backend::Vulkan, Backend::Dx11, Backend::Dx12][..]
    } else if cfg!(target_os = "macos") {
        &[Backend::Gl, Backend::Vulkan, Backend::Metal][..]
    } else {
        &[Backend::Gl, Backend::Vulkan][..]
    };
    let backend = PickList::new(
        backend,
        backends,
        Some(board_state.backend_temp),
        Message::BackendChanged,
    );

    let apply = Row::new()
        .push(
            Button::new(apply_button, crate::style::icon::Icon::enter())
                .style(Symbol)
                .on_press(Message::Apply),
        )
        .align_items(iced::Align::End);
    setting_column
        .align_items(iced::Align::Center)
        .height(iced::Length::Fill)
        .width(iced::Length::FillPortion(10))
        .push(language)
        .push(backend)
        .push(apply)
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
    // TODO:增加可更改地址
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
            backend: Backend::default(),
        }
    }
}
