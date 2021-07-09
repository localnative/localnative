use std::path::PathBuf;

use crate::{error_handle, style::Theme, translate::Language};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

// config 需要保存主题、语言、limit、disable_delete_tip、is_first_open
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub theme: Theme,
    pub language: Language,
    pub limit: u32,
    pub disable_delete_tip: bool,
    pub is_first_open: bool,
    pub date_filter_is_show: bool,
    pub date_mode_is_full: bool,
    pub day_uw: f32,
    pub month_uw: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            language: Language::English,
            limit: 10,
            disable_delete_tip: false,
            is_first_open: true,
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
    pub async fn load() -> Option<Self> {
        let mut contents = String::new();
        let path = Self::config_path();

        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(error_handle)
            .ok()?;

        file.read_to_string(&mut contents)
            .await
            .map_err(error_handle)
            .ok()?;

        serde_json::from_str(&contents).map_err(error_handle).ok()
    }
    pub fn sync_load() -> Option<Self> {
        tokio::runtime::Runtime::new()
            .map_err(error_handle)
            .ok()?
            .block_on(Self::load())
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

    {
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
    }
    Some(())
}
