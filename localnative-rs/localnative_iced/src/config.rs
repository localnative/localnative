use core::ops::Not;
use std::path::PathBuf;

use crate::{error_handle, translate::Language};
use plotters::style::RGBColor;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ThemeType {
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
}

impl ToString for ThemeType {
    fn to_string(&self) -> String {
        match self {
            ThemeType::Light => "Light".to_string(),
            ThemeType::Dark => "Dark".to_string(),
            ThemeType::Dracula => "Dracula".to_string(),
            ThemeType::Nord => "Nord".to_string(),
            ThemeType::SolarizedLight => "SolarizedLight".to_string(),
            ThemeType::SolarizedDark => "SolarizedDark".to_string(),
            ThemeType::GruvboxLight => "GruvboxLight".to_string(),
            ThemeType::GruvboxDark => "GruvboxDark".to_string(),
            ThemeType::CatppuccinLatte => "CatppuccinLatte".to_string(),
            ThemeType::CatppuccinFrappe => "CatppuccinFrappe".to_string(),
            ThemeType::CatppuccinMacchiato => "CatppuccinMacchiato".to_string(),
            ThemeType::CatppuccinMocha => "CatppuccinMocha".to_string(),
            ThemeType::TokyoNight => "TokyoNight".to_string(),
            ThemeType::TokyoNightStorm => "TokyoNightStorm".to_string(),
            ThemeType::TokyoNightLight => "TokyoNightLight".to_string(),
            ThemeType::KanagawaWave => "KanagawaWave".to_string(),
            ThemeType::KanagawaDragon => "KanagawaDragon".to_string(),
            ThemeType::KanagawaLotus => "KanagawaLotus".to_string(),
            ThemeType::Moonfly => "Moonfly".to_string(),
            ThemeType::Nightfly => "Nightfly".to_string(),
            ThemeType::Oxocarbon => "Oxocarbon".to_string(),
        }
    }
}

impl ThemeType {
    pub const ALL_LIGHT: &'static [Self] = &[
        Self::Light,
        Self::SolarizedLight,
        Self::GruvboxLight,
        Self::CatppuccinLatte,
        Self::TokyoNightLight,
        Self::KanagawaLotus,
    ];
    pub const ALL_DARK: &'static [Self] = &[
        Self::Dark,
        Self::Dracula,
        Self::Nord,
        Self::SolarizedDark,
        Self::GruvboxDark,
        Self::CatppuccinFrappe,
        Self::CatppuccinMacchiato,
        Self::CatppuccinMocha,
        Self::TokyoNight,
        Self::TokyoNightStorm,
        Self::KanagawaWave,
        Self::KanagawaDragon,
        Self::Moonfly,
        Self::Nightfly,
        Self::Oxocarbon,
    ];

    fn from_config(config: &Config) -> Self {
        let kind = config.theme_kind;
        match kind {
            ThemeKind::Light => config.light_theme,
            ThemeKind::Dark => config.dark_theme,
        }
    }

    pub fn selection_color(&self) -> RGBColor {
        match self {
            Self::Light => RGBColor(255, 204, 128),              // 橙色
            Self::Dark => RGBColor(255, 153, 51),                // 橙黄色
            Self::Dracula => RGBColor(189, 147, 249),            // 紫色
            Self::Nord => RGBColor(143, 188, 187),               // 青色
            Self::SolarizedLight => RGBColor(253, 246, 227),     // 浅黄色
            Self::SolarizedDark => RGBColor(0, 43, 54),          // 深绿色
            Self::GruvboxLight => RGBColor(253, 246, 227),       // 浅黄色
            Self::GruvboxDark => RGBColor(255, 153, 51),         // 橙黄色
            Self::CatppuccinLatte => RGBColor(253, 246, 227),    // 浅黄色
            Self::CatppuccinFrappe => RGBColor(255, 153, 51),    // 橙黄色
            Self::CatppuccinMacchiato => RGBColor(255, 153, 51), // 橙黄色
            Self::CatppuccinMocha => RGBColor(255, 153, 51),     // 橙黄色
            Self::TokyoNight => RGBColor(189, 147, 249),         // 紫色
            Self::TokyoNightStorm => RGBColor(189, 147, 249),    // 紫色
            Self::TokyoNightLight => RGBColor(253, 246, 227),    // 浅黄色
            Self::KanagawaWave => RGBColor(143, 188, 187),       // 青色
            Self::KanagawaDragon => RGBColor(143, 188, 187),     // 青色
            Self::KanagawaLotus => RGBColor(143, 188, 187),      // 青色
            Self::Moonfly => RGBColor(189, 147, 249),            // 紫色
            Self::Nightfly => RGBColor(189, 147, 249),           // 紫色
            Self::Oxocarbon => RGBColor(189, 147, 249),          // 紫色
        }
    }

    pub fn line_color(&self) -> RGBColor {
        match self {
            Self::Light => RGBColor(0, 128, 255),               // 蓝色
            Self::Dark => RGBColor(255, 102, 0),                // 橙红色
            Self::Dracula => RGBColor(255, 121, 198),           // 粉色
            Self::Nord => RGBColor(136, 192, 208),              // 浅青色
            Self::SolarizedLight => RGBColor(38, 139, 210),     // 蓝色
            Self::SolarizedDark => RGBColor(131, 148, 150),     // 灰色
            Self::GruvboxLight => RGBColor(38, 139, 210),       // 蓝色
            Self::GruvboxDark => RGBColor(255, 102, 0),         // 橙红色
            Self::CatppuccinLatte => RGBColor(38, 139, 210),    // 蓝色
            Self::CatppuccinFrappe => RGBColor(255, 102, 0),    // 橙红色
            Self::CatppuccinMacchiato => RGBColor(255, 102, 0), // 橙红色
            Self::CatppuccinMocha => RGBColor(255, 102, 0),     // 橙红色
            Self::TokyoNight => RGBColor(255, 121, 198),        // 粉色
            Self::TokyoNightStorm => RGBColor(255, 121, 198),   // 粉色
            Self::TokyoNightLight => RGBColor(38, 139, 210),    // 蓝色
            Self::KanagawaWave => RGBColor(136, 192, 208),      // 浅青色
            Self::KanagawaDragon => RGBColor(136, 192, 208),    // 浅青色
            Self::KanagawaLotus => RGBColor(136, 192, 208),     // 浅青色
            Self::Moonfly => RGBColor(255, 121, 198),           // 粉色
            Self::Nightfly => RGBColor(255, 121, 198),          // 粉色
            Self::Oxocarbon => RGBColor(255, 121, 198),         // 粉色
        }
    }

    pub fn fill_color(&self) -> RGBColor {
        match self {
            Self::Light => RGBColor(204, 229, 255),            // 淡蓝色
            Self::Dark => RGBColor(80, 80, 80),                // 深灰色
            Self::Dracula => RGBColor(40, 42, 54),             // 深紫色
            Self::Nord => RGBColor(236, 239, 244),             // 浅灰色
            Self::SolarizedLight => RGBColor(253, 246, 227),   // 浅黄色
            Self::SolarizedDark => RGBColor(7, 54, 66),        // 深蓝色
            Self::GruvboxLight => RGBColor(253, 246, 227),     // 浅黄色
            Self::GruvboxDark => RGBColor(80, 80, 80),         // 深灰色
            Self::CatppuccinLatte => RGBColor(253, 246, 227),  // 浅黄色
            Self::CatppuccinFrappe => RGBColor(80, 80, 80),    // 深灰色
            Self::CatppuccinMacchiato => RGBColor(80, 80, 80), // 深灰色
            Self::CatppuccinMocha => RGBColor(80, 80, 80),     // 深灰色
            Self::TokyoNight => RGBColor(40, 42, 54),          // 深紫色
            Self::TokyoNightStorm => RGBColor(40, 42, 54),     // 深紫色
            Self::TokyoNightLight => RGBColor(253, 246, 227),  // 浅黄色
            Self::KanagawaWave => RGBColor(236, 239, 244),     // 浅灰色
            Self::KanagawaDragon => RGBColor(236, 239, 244),   // 浅灰色
            Self::KanagawaLotus => RGBColor(236, 239, 244),    // 浅灰色
            Self::Moonfly => RGBColor(40, 42, 54),             // 深紫色
            Self::Nightfly => RGBColor(40, 42, 54),            // 深紫色
            Self::Oxocarbon => RGBColor(40, 42, 54),           // 深紫色
        }
    }

    pub fn text_color(&self) -> RGBColor {
        match self {
            Self::Light => RGBColor(0, 0, 0),                     // 黑色
            Self::Dark => RGBColor(255, 255, 255),                // 白色
            Self::Dracula => RGBColor(248, 248, 242),             // 浅灰色
            Self::Nord => RGBColor(46, 52, 64),                   // 深灰色
            Self::SolarizedLight => RGBColor(101, 123, 131),      // 灰色
            Self::SolarizedDark => RGBColor(131, 148, 150),       // 灰色
            Self::GruvboxLight => RGBColor(101, 123, 131),        // 灰色
            Self::GruvboxDark => RGBColor(255, 255, 255),         // 白色
            Self::CatppuccinLatte => RGBColor(101, 123, 131),     // 灰色
            Self::CatppuccinFrappe => RGBColor(255, 255, 255),    // 白色
            Self::CatppuccinMacchiato => RGBColor(255, 255, 255), // 白色
            Self::CatppuccinMocha => RGBColor(255, 255, 255),     // 白色
            Self::TokyoNight => RGBColor(248, 248, 242),          // 浅灰色
            Self::TokyoNightStorm => RGBColor(248, 248, 242),     // 浅灰色
            Self::TokyoNightLight => RGBColor(101, 123, 131),     // 灰色
            Self::KanagawaWave => RGBColor(46, 52, 64),           // 深灰色
            Self::KanagawaDragon => RGBColor(46, 52, 64),         // 深灰色
            Self::KanagawaLotus => RGBColor(46, 52, 64),          // 深灰色
            Self::Moonfly => RGBColor(248, 248, 242),             // 浅灰色
            Self::Nightfly => RGBColor(248, 248, 242),            // 浅灰色
            Self::Oxocarbon => RGBColor(248, 248, 242),           // 浅灰色
        }
    }
}

impl Into<iced::Theme> for ThemeType {
    fn into(self) -> iced::Theme {
        match self {
            ThemeType::Light => iced::Theme::Light,
            ThemeType::Dark => iced::Theme::Dark,
            ThemeType::Dracula => iced::Theme::Dracula,
            ThemeType::Nord => iced::Theme::Nord,
            ThemeType::SolarizedLight => iced::Theme::SolarizedLight,
            ThemeType::SolarizedDark => iced::Theme::SolarizedDark,
            ThemeType::GruvboxLight => iced::Theme::GruvboxLight,
            ThemeType::GruvboxDark => iced::Theme::GruvboxDark,
            ThemeType::CatppuccinLatte => iced::Theme::CatppuccinLatte,
            ThemeType::CatppuccinFrappe => iced::Theme::CatppuccinFrappe,
            ThemeType::CatppuccinMacchiato => iced::Theme::CatppuccinMacchiato,
            ThemeType::CatppuccinMocha => iced::Theme::CatppuccinMocha,
            ThemeType::TokyoNight => iced::Theme::TokyoNight,
            ThemeType::TokyoNightStorm => iced::Theme::TokyoNightStorm,
            ThemeType::TokyoNightLight => iced::Theme::TokyoNightLight,
            ThemeType::KanagawaWave => iced::Theme::KanagawaWave,
            ThemeType::KanagawaDragon => iced::Theme::KanagawaDragon,
            ThemeType::KanagawaLotus => iced::Theme::KanagawaLotus,
            ThemeType::Moonfly => iced::Theme::Moonfly,
            ThemeType::Nightfly => iced::Theme::Nightfly,
            ThemeType::Oxocarbon => iced::Theme::Oxocarbon,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize, Serialize, Clone, Copy)]
pub enum ThemeKind {
    #[default]
    Light,
    Dark,
}

impl Not for ThemeKind {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            ThemeKind::Light => ThemeKind::Dark,
            ThemeKind::Dark => ThemeKind::Light,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub light_theme: ThemeType,
    pub dark_theme: ThemeType,
    pub theme_kind: ThemeKind,
    pub language: Language,
    pub limit: u32,
    pub disable_delete_tip: bool,
    pub date_filter_is_show: bool,
    pub date_mode_is_full: bool,
    pub day_uw: f32,
    pub month_uw: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            light_theme: ThemeType::Light,
            dark_theme: ThemeType::Dark,
            theme_kind: ThemeKind::Light,
            language: Language::English,
            limit: 10,
            disable_delete_tip: false,
            date_filter_is_show: true,
            date_mode_is_full: true,
            day_uw: 17.0,
            month_uw: 17.0,
        }
    }
}

impl Config {
    pub const APP_NAME: &'static str = "LocalNative";
    pub const CONFIG_NAME: &'static str = "config.json";
    pub fn app_dir() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(Self::APP_NAME)
        } else {
            std::env::temp_dir().join(Self::APP_NAME)
        }
    }
    pub fn config_path() -> PathBuf {
        Self::app_dir().join(Self::CONFIG_NAME)
    }
    pub fn load() -> Option<Self> {
        use std::io::Read;
        let mut contents = String::new();
        let path = Self::config_path();

        let mut file = std::fs::File::open(path).map_err(error_handle).ok()?;

        file.read_to_string(&mut contents)
            .map_err(error_handle)
            .ok()?;

        serde_json::from_str(&contents).map_err(error_handle).ok()
    }
    pub fn theme(&self) -> ThemeType {
        ThemeType::from_config(self)
    }
}

pub async fn save(json: String) -> Option<()> {
    use tokio::io::AsyncWriteExt;

    println!("json:{}", json);
    let raw_data = json.as_bytes();
    let path = Config::config_path();

    if let Some(dir) = path.parent() {
        if !dir.exists() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(error_handle)
                .ok()?;
        }
    }

    if path.is_dir() {
        tokio::fs::remove_dir(&path)
            .await
            .map_err(error_handle)
            .ok()?;
    }
    let mut file = tokio::fs::File::create(&path)
        .await
        .map_err(error_handle)
        .ok()?;

    file.write_all(raw_data).await.map_err(error_handle).ok()?;

    Some(())
}
