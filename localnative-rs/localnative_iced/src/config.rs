use std::fmt::Display;

use directories_next::BaseDirs;
use iced::{pick_list, slider, Column, Element, PickList, Slider};
use serde::{Deserialize, Serialize};

use crate::style::Theme;

#[derive(Debug, Clone)]
pub enum Message {
    LimitChanged(u32),
    LanguageChanged(Language),
    ThemeChanged(Theme),
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Config {
    pub limit: u32,
    pub language: Language,
    pub theme: Theme,
    pub is_created_db: bool,
}
impl Config {
    pub async fn new() -> Self {
        Config::default()
    }
}
#[derive(Debug, Clone, Default)]
pub struct ConfigView {
    pub config: Config,
    pub state: State,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    limit: slider::State,
    offset: slider::State,
    language: pick_list::State<Language>,
    theme: pick_list::State<Theme>,
}

impl ConfigView {
    pub fn limit(&self) -> u32 {
        self.config.limit
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::LimitChanged(limit) => {
                self.config.limit = limit;
            }
            Message::LanguageChanged(l) => {
                // TODO:做出实质的语言更改
                self.config.language = l;
            }
            Message::ThemeChanged(t) => {
                //TODO：做出实质的语言更改
                self.config.theme = t;
            }
        }
    }
    pub fn viwe(&mut self) -> Element<Message> {
        let ConfigView { config, state } = self;
        let State {
            limit,
            language,
            theme,
            ..
        } = state;
        let limit = Slider::new(limit, 5..=50, config.limit, Message::LimitChanged);
        let language = PickList::new(
            language,
            &[Language::Chinese, Language::English][..],
            Some(config.language.clone()),
            Message::LanguageChanged,
        );
        let theme = PickList::new(
            theme,
            &[Theme::Dark, Theme::Light][..],
            Some(config.theme.clone()),
            Message::ThemeChanged,
        );
        iced::Container::new(
            Column::new()
                .align_items(iced::Align::Center)
                .spacing(10)
                .push(limit)
                .push(language)
                .push(theme),
        )
        .width(iced::Length::FillPortion(1))
        .align_x(iced::Align::Start)
        .align_y(iced::Align::Center)
        .into()
    }
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
        std::env::current_dir().unwrap().join("LocalNative")
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
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
