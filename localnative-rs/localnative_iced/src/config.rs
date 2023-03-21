use core::ops::Not;
use std::path::PathBuf;

use crate::{error_handle, translate::Language};
use plotters::style::RGBColor;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ThemeType {
    Light,
    Dark,
    // Custom,
}

impl ThemeType {
    pub fn selection_color(&self) -> RGBColor {
        match self {
            Self::Light => RGBColor(255, 204, 128), // 橙色
            Self::Dark => RGBColor(255, 153, 51),   // 橙黄色
        }
    }

    pub fn line_color(&self) -> RGBColor {
        match self {
            Self::Light => RGBColor(0, 128, 255), // 蓝色
            Self::Dark => RGBColor(255, 102, 0),  // 橙红色
        }
    }

    pub fn fill_color(&self) -> RGBColor {
        match self {
            Self::Light => RGBColor(204, 229, 255), // 淡蓝色
            Self::Dark => RGBColor(80, 80, 80),     // 深灰色
        }
    }

    pub fn text_color(&self) -> RGBColor {
        match self {
            Self::Light => RGBColor(0, 0, 0),      // 黑色
            Self::Dark => RGBColor(255, 255, 255), // 白色
        }
    }
}

impl Not for ThemeType {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            ThemeType::Light => ThemeType::Dark,
            ThemeType::Dark => ThemeType::Light,
        }
    }
}

impl Into<iced::Theme> for ThemeType {
    fn into(self) -> iced::Theme {
        match self {
            ThemeType::Light => iced::Theme::Light,
            ThemeType::Dark => iced::Theme::Dark,
        }
    }
}

// config 需要保存主题、语言、limit、disable_delete_tip、is_first_open
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub theme: ThemeType,
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
            theme: ThemeType::Light,
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
        if let Some(home) = localnative_core::dirs::home_dir() {
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
