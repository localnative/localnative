use directories_next::BaseDirs;
use iced::{
    button, pick_list, qr_code, slider, text_input, Button, Column, Element, PickList, Row, Rule,
    Slider, Text, TextInput,
};
#[cfg(feature = "wgpu")]
use std::fmt::Display;
use std::{net::SocketAddr, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    args,
    localization::Language,
    logger::{Logger, Record},
    style::{
        symbol::{self, Symbol},
        Theme,
    },
    tr,
};

#[derive(Debug, Clone)]
pub enum Message {
    LimitChanged(u32),
    LanguageChanged(Language),
    #[cfg(feature = "wgpu")]
    BackendChanged(Backend),
    AddrsChanged(String),
    OpenFile,
    SelectSettingBoard,
    Server,
    Sync,
    ThemeChanged,
    OpenSlider,
    HideSlider,
    ApplySave,
    Reset,
    ClearAddrInput,
    FixHost,
    Empty,
    BackContent,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Config {
    pub limit: u32,
    pub language: Language,
    pub theme: Theme,
    pub is_created_db: bool,
}
#[cfg(feature = "wgpu")]
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
#[cfg(feature = "wgpu")]
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
#[cfg(feature = "wgpu")]
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
#[cfg(feature = "wgpu")]
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
    back_content_button: button::State,
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
    save_button: button::State,
    reset_button: button::State,
    fix_web_host_button: button::State,
    language: pick_list::State<Language>,
    language_temp: Language,
    #[cfg(feature = "wgpu")]
    backend: pick_list::State<Backend>,
    #[cfg(feature = "wgpu")]
    pub backend_temp: Backend,
    #[cfg(feature = "wgpu")]
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
            let data = tr!("get-ip-fail");
            qr_code::State::new(data.to_string()).unwrap_or_else(|e| {
                qr_code::State::new(format!("Error in qrcode generation: {:?}", e)).unwrap()
            })
        };
        #[cfg(feature = "wgpu")]
        let backend = Backend::from_env();
        Self {
            config,
            board_state: BoardState {
                limit_temp: config.limit,
                language_temp: config.language,
                is_open: false,
                #[cfg(feature = "wgpu")]
                backend_temp: backend,
                #[cfg(feature = "wgpu")]
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
            Message::ApplySave => {
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
            #[cfg(feature = "wgpu")]
            Message::BackendChanged(backend) => {
                self.board_state.backend_temp = backend;
            }
            Message::Reset => {
                #[cfg(feature = "wgpu")]
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
            Message::Empty => {}
            Message::BackContent => {}
        }
    }
    pub fn viwe<'a>(
        &'a mut self,
        server_state: bool,
        logger: &'a mut Option<Logger>,
    ) -> Element<'a, Message> {
        let SettingView { config, state, .. } = self;
        left_bar_viwe(state, config.theme, server_state, logger)
    }
    pub fn setting_board_open_view<'a>(
        &'a mut self,
        server_state: bool,
        logger: &'a mut Option<Logger>,
    ) -> Element<'a, Message> {
        let SettingView {
            config,
            state,
            board_state,
            ..
        } = self;
        let left_bar = left_bar_viwe(state, config.theme, server_state, logger);
        let setting_board = setting_board_view(board_state, &*config);
        Row::new().push(left_bar).push(setting_board).into()
    }
    pub fn sync_board_open_view<'a>(
        &'a mut self,
        server_state: bool,
        logger: &'a mut Option<Logger>,
    ) -> Element<'a, Message> {
        let SettingView {
            config,
            state,
            sync_state,
            ..
        } = self;
        let left_bar = left_bar_viwe(state, config.theme, server_state, logger);
        let sync_board = sync_board_view(&*sync_state);
        Row::new().push(left_bar).push(sync_board).into()
    }
}
fn sync_board_view(state: &SyncState) -> Element<Message> {
    let SyncState { qr_code, qr_data } = state;
    let args = args!("ip"=>qr_data.clone());
    Column::new()
        .height(iced::Length::Fill)
        .width(iced::Length::FillPortion(10))
        .align_items(iced::Align::Center)
        .push(Text::new(tr!("server-p0")))
        .push(Text::new(tr!(
            "server-p1";
            &args
        )))
        .push(Text::new(tr!("server-p3")))
        .push(iced::QRCode::new(qr_code).cell_size(10))
        .into()
}

fn left_bar_viwe<'a>(
    state: &'a mut State,
    theme: Theme,
    server_state: bool,
    logger: &'a mut Option<Logger>,
) -> Element<'a, Message> {
    let State {
        theme_button,
        setting_button,
        server_button,
        open_file_button,
        clear_button,
        addr_input,
        addr,
        back_content_button,
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
    let server_button =
        Button::new(server_button, Text::new(tr!("server"))).on_press(Message::Server);
    let clear_button = Button::new(clear_button, crate::style::icon::Icon::close(18))
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
        .push(Text::new(tr!("start-sync")))
        .push(Row::new().push(addr_input).push(clear_button))
        .push(Text::new(tr!("input-tip")));
    let open_file_button =
        Button::new(open_file_button, Text::new(tr!("sync-via-file"))).on_press(Message::OpenFile);
    let setting_button = Button::new(
        setting_button,
        Row::new()
            .push(crate::style::icon::Icon::settings())
            .push(Text::new(tr!("setting"))),
    )
    .style(Symbol)
    .on_press(Message::SelectSettingBoard);
    let back_button =
        Button::new(back_content_button, Text::new(tr!("back"))).on_press(Message::BackContent);
    let server_state = if server_state {
        tr!("server-opened")
    } else {
        tr!("server-closed")
    };
    let mut content = Column::new()
        .align_items(iced::Align::Center)
        .height(iced::Length::Fill)
        .width(iced::Length::FillPortion(2))
        .spacing(10)
        .push(open_file_button)
        .push(server_button)
        .push(Text::new(server_state))
        .push(addr_row);
    content = if let Some(ref mut logger) = logger {
        content.push(Record::new(logger.state()))
    } else {
        content
    };
    content
        .push(Rule::vertical(50).style(symbol::Symbol))
        .push(back_button)
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
        save_button,
        language,
        #[cfg(feature = "wgpu")]
        backend,
        reset_button,
        fix_web_host_button,
        ..
    } = board_state;
    let args = args!("num"=>board_state.limit_temp);
    let limit_text = Text::new(tr!("limit";&args));
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
        .push(Text::new(tr!("language")))
        .push(PickList::new(
            language,
            &[Language::Chinese, Language::English][..],
            Some(board_state.language_temp),
            Message::LanguageChanged,
        ));
    #[cfg(feature = "wgpu")]
    let backends = {
        #[cfg(target_os = "windows")]
        let res = &[Backend::Gl, Backend::Vulkan, Backend::Dx11, Backend::Dx12][..];
        #[cfg(target_os = "macos")]
        let res = &[Backend::Gl, Backend::Vulkan, Backend::Metal][..];
        #[cfg(target_os = "linux")]
        let res = &[Backend::Gl, Backend::Vulkan][..];
        res
    };
    #[cfg(feature = "wgpu")]
    let backend = Row::new()
        .spacing(300)
        .push(Text::new(tr!("render-backend")))
        .push(PickList::new(
            backend,
            backends,
            Some(board_state.backend_temp),
            Message::BackendChanged,
        ));
    let is_need_apply;
    #[cfg(feature = "wgpu")]
    {
        is_need_apply = config.language != board_state.language_temp
            || config.limit != board_state.limit_temp
            || board_state.backend_org != board_state.backend_temp;
    }
    #[cfg(feature = "opengl")]
    {
        is_need_apply =
            config.language != board_state.language_temp || config.limit != board_state.limit_temp;
    }
    let apply = if is_need_apply {
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
                            .push(Text::new(tr!("reset"))),
                    )
                    .style(symbol::Symbol)
                    .on_press(Message::Reset),
                )
                .push(
                    Button::new(
                        save_button,
                        Row::new()
                            .align_items(iced::Align::Center)
                            .push(crate::style::icon::Icon::enter())
                            .push(Text::new(tr!("save-setting"))),
                    )
                    .style(symbol::Symbol)
                    .on_press(Message::ApplySave),
                ),
        )
    } else {
        None
    };
    let fix_button =
        Button::new(fix_web_host_button, Text::new(tr!("fix-ext-host"))).on_press(Message::FixHost);
    let res = setting_column
        .align_items(iced::Align::Center)
        .height(iced::Length::Fill)
        .width(iced::Length::FillPortion(10))
        .push(language);
    #[cfg(feature = "wgpu")]
    let res = res.push(backend);
    let res = res
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
