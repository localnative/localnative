use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize)]
pub struct AppHost {
    name: String,
    description: String,
    path: PathBuf,
    #[serde(rename = "type")]
    tp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_origins: Option<Vec<String>>,
}

impl AppHost {
    pub fn path() -> PathBuf {
        let name = {
            #[cfg(target_os = "windows")]
            {
                "localnative-web-ext-host.exe"
            }
            #[cfg(not(target_os = "windows"))]
            {
                "localnative-web-ext-host"
            }
        };
        #[cfg(not(target_os = "linux"))]
        let mut path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();

        #[cfg(target_os = "linux")]
        let mut path = localnative_core::dirs::home_dir()
            .unwrap()
            .join("LocalNative");

        path = path.join(name);
        println!("path : {:?}", path);
        path
    }

    pub fn firefox() -> Self {
        Self {
            name: "app.localnative".to_owned(),
            description: "Local Native Host".to_owned(),
            path: Self::path(),
            tp: "stdio".to_owned(),
            allowed_extensions: Some(vec!["localnative@example.org".to_owned()]),
            allowed_origins: None,
        }
    }
    pub fn chrome() -> Self {
        Self {
            name: "app.localnative".to_owned(),
            description: "Local Native Host".to_owned(),
            path: Self::path(),
            tp: "stdio".to_owned(),
            allowed_extensions: None,
            allowed_origins: Some(vec![
                "chrome-extension://oclkmkeameccmgnajgogjlhdjeaconnb/".to_owned()
            ]),
        }
    }
    pub fn raw_data(&self) -> Vec<u8> {
        println!("{:?}", serde_json::to_string_pretty(self));
        serde_json::to_vec(self).unwrap()
    }
}

//     Windows Registry Editor Version 5.00
// [HKEY_CURRENT_USER\Software\Google\Chrome\NativeMessagingHosts\app.localnative]
// @="PATH_TO_CHROME_MANIFEST\\app.localnative.json"
//let hkey = winreg::RegKey::predef(HKEY_CURRENT_USER);

// [HKEY_CURRENT_USER\Software\Chromium\NativeMessagingHosts\app.localnative]
// @="PATH_TO_CHROME_MANIFEST\\app.localnative.json"

// [HKEY_CURRENT_USER\Software\Mozilla\NativeMessagingHosts\app.localnative]
// @="PATH_TO_FIREFOX_MANIFEST\\app.localnative.json"
#[cfg(target_os = "windows")]
fn registr(kind: WebKind) {
    use winreg::enums::*;

    use crate::error_handle;
    let path = kind.registr_path();
    let write_path = path.join("app.localnative");
    let json_path = kind.json_path().unwrap();
    let key = winreg::RegKey::predef(HKEY_CURRENT_USER);
    let value = key
        .open_subkey(&path)
        .and_then(|k| k.open_subkey(Path::new("app.localnative")))
        .map_err(error_handle)
        .and_then(|k| k.get_value::<String, &str>("").map_err(error_handle))
        .ok();
    if let Some(v) = value {
        if v != json_path {
            key.open_subkey_with_flags(&write_path, KEY_WRITE)
                .map_err(error_handle)
                .and_then(|writer| writer.set_value("", &json_path).map_err(error_handle))
                .unwrap();
        }
    } else {
        key.create_subkey_with_flags(&write_path, KEY_WRITE)
            .and_then(|(writer, _)| writer.set_value("", &json_path))
            .map_err(error_handle)
            .unwrap();
    }
}

pub fn firefox_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home_dir| {
        #[cfg(target_os = "macos")]
        {
            home_dir
                .join("Library")
                .join("Application Support")
                .join("Mozilla")
        }
        #[cfg(target_os = "linux")]
        {
            home_dir.join(".mozilla")
        }
        #[cfg(target_os = "windows")]
        {
            home_dir.join("LocalNative").join("config").join("mozilla")
        }
    })
}

pub fn chrome_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home_dir| {
        #[cfg(target_os = "macos")]
        {
            home_dir
                .join("Library")
                .join("Application Support")
                .join("Google")
                .join("Chrome")
        }
        #[cfg(target_os = "linux")]
        {
            home_dir.join(".config").join("google-chrome")
        }
        #[cfg(target_os = "windows")]
        {
            home_dir.join("LocalNative").join("config").join("chrome")
        }
    })
}

#[derive(Debug, Clone, Copy)]
pub enum WebKind {
    FireFox,
    Chrome,
}

impl WebKind {
    pub async fn init_all() {
        #[cfg(target_os = "linux")]
        {
            let from = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("localnative-web-ext-host");
            let to = localnative_core::dirs::home_dir()
                .unwrap()
                .join("LocalNative")
                .join("localnative-web-ext-host");

            if to.exists() && to.is_dir() {
                tokio::fs::remove_dir(&to).await.unwrap();
            }

            tokio::fs::copy(from, to).await.unwrap();
        }
        tokio::join!(try_init_file(Self::FireFox), try_init_file(Self::Chrome),);
    }
    #[cfg(target_os = "windows")]
    fn registr_path(&self) -> PathBuf {
        match self {
            WebKind::FireFox => Path::new("Software")
                .join("Mozilla")
                .join("NativeMessagingHosts"),
            WebKind::Chrome => Path::new("Software")
                .join("Google")
                .join("Chrome")
                .join("NativeMessagingHosts"),
        }
    }

    fn path(&self) -> Option<PathBuf> {
        match self {
            WebKind::FireFox => firefox_path(),
            WebKind::Chrome => chrome_path(),
        }
        .map(|browser_path| {
            #[cfg(target_os = "macos")]
            {
                browser_path.join("NativeMessagingHosts")
            }
            #[cfg(target_os = "linux")]
            match self {
                WebKind::FireFox => browser_path.join("native-messaging-hosts"),
                _ => browser_path.join("NativeMessagingHosts"),
            }
            #[cfg(target_os = "windows")]
            browser_path
        })
    }
    fn host(&self) -> AppHost {
        match self {
            WebKind::FireFox => AppHost::firefox(),
            WebKind::Chrome => AppHost::chrome(),
        }
    }
    #[cfg(target_os = "windows")]
    fn json_path(&self) -> Option<String> {
        let path = self.path()?.join("app.localnative.json");
        path.into_os_string().into_string().ok()
    }
}
async fn try_init_file(kind: WebKind) {
    if let Some(dir_path) = kind.path() {
        #[cfg(target_os = "windows")]
        registr(kind);
        let raw_file = kind.host().raw_data();
        init_file(&dir_path, &raw_file).await.unwrap();
    }
}

async fn init_file(dir_path: &Path, raw_file: &[u8]) -> std::io::Result<()> {
    let file_path = dir_path.join("app.localnative.json");
    if file_path.exists() {
        if file_path.is_file() {
            let file = tokio::fs::read(&file_path).await?;
            if file != *raw_file {
                tokio::fs::write(&file_path, raw_file).await?;
            }
        } else {
            tokio::fs::remove_dir(&file_path).await?;
            create_and_write_file(dir_path, &file_path, raw_file).await?;
        }
    } else {
        create_and_write_file(dir_path, &file_path, raw_file).await?;
    }

    Ok(())
}

async fn create_and_write_file(
    dir_path: &Path,
    file_path: &Path,
    raw_file: &[u8],
) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dir_path).await?;
    tokio::fs::write(file_path, raw_file).await?;
    Ok(())
}
