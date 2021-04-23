use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize)]
pub struct AppHost {
    name: String,
    description: String,
    path: String,
    #[serde(rename = "type")]
    tp: String,
    allowed_extensions: Vec<String>,
}
impl AppHost {
    pub fn path() -> anyhow::Result<String> {
        let mut name = String::from("localnative-web-ext-host");
        // TODO: add version
        // name += "-0.4.2";
        if cfg!(windows) {
            name += ".exe";
        };
        let mut path = std::env::current_dir()?;
        path = path.join(name);
        let mut res = path
            .into_os_string()
            .into_string()
            .map_err(|err| anyhow::anyhow!("faid to get path {:?}", err))?;
        res = if cfg!(windows) {
            res.replace("//", "/")
        } else {
            res
        };
        Ok(res)
    }
    pub fn firefox() -> anyhow::Result<Self> {
        Ok(Self {
            name: "app.localnative".to_owned(),
            description: "Local Native Host".to_owned(),
            path: Self::path()?,
            tp: "stdio".to_owned(),
            allowed_extensions: vec!["localnative@example.org".to_owned()],
        })
    }
    pub fn chrome() -> anyhow::Result<Self> {
        Ok(Self {
            name: "app.localnative".to_owned(),
            description: "Local Native Host".to_owned(),
            path: Self::path()?,
            tp: "stdio".to_owned(),
            allowed_extensions: vec![
                "chrome-extension://oclkmkeameccmgnajgogjlhdjeaconnb/".to_owned()
            ],
        })
    }
    pub fn raw_data(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
}

pub async fn init_app_host() -> anyhow::Result<()> {
    WebKind::init_all().await
}

//     Windows Registry Editor Version 5.00
// [HKEY_CURRENT_USER\Software\Google\Chrome\NativeMessagingHosts\app.localnative]
// @="PATH_TO_CHROME_MANIFEST\\app.localnative.json"
//let hkey = winreg::RegKey::predef(HKEY_CURRENT_USER);

// [HKEY_CURRENT_USER\Software\Chromium\NativeMessagingHosts\app.localnative]
// @="PATH_TO_CHROME_MANIFEST\\app.localnative.json"

// [HKEY_CURRENT_USER\Software\Mozilla\NativeMessagingHosts\app.localnative]
// @="PATH_TO_FIREFOX_MANIFEST\\app.localnative.json"
#[cfg(windows)]
fn registr(kind: WebKind) -> anyhow::Result<Option<WebKind>> {
    use winreg::enums::*;
    log::info!("registr starting");
    let path = kind.registr_path();
    let write_path = path.join("app.localnative");
    log::info!("registr write path:{:?}", &write_path);
    log::info!("registr get kind host");
    let json_path = kind.json_path()?;
    let key = winreg::RegKey::predef(HKEY_CURRENT_USER);

    if let Ok(k) = key.open_subkey(&path) {
        log::info!("registr open subkey success");
        if let Ok(k) = k.open_subkey(Path::new("app.localnative")) {
            if let Ok(v) = k.get_value::<String, &str>("") {
                log::info!("registr get value success:{:?}", &v);
                if v != json_path {
                    log::info!("registr start set value");
                    let writer = key.open_subkey_with_flags(&write_path, KEY_WRITE)?;
                    writer.set_value("", &json_path)?;
                    log::info!("registr set value ok");
                    return Ok(Some(kind));
                } else {
                    log::info!("registr value Eq");
                    return Ok(Some(kind));
                }
            }else {
                log::error!("get registr value fail!");
            }
        } else {
            log::info!("registr start set value");
            let (writer, _disposition) = key.create_subkey_with_flags(&write_path, KEY_WRITE)?;
            writer.set_value("", &json_path)?;
            log::info!("registr set value ok");
            return Ok(Some(kind));
        }
    }
    Ok(None)
}

pub fn firefox_path() -> anyhow::Result<PathBuf> {
    if let Some(user_dir) = directories_next::UserDirs::new() {
        let home_dir = user_dir.home_dir();
        if cfg!(target_os = "macos") {
            Ok(home_dir
                .join("Library")
                .join("Application Support")
                .join("Mozilla")
                .join("NativeMessagingHosts"))
        } else if cfg!(target_os = "linux") {
            Ok(home_dir.join(".mozilla").join("native-messaging-hosts"))
        } else if cfg!(target_os = "windows") {
            Ok(home_dir.join("LocalNative").join("config").join("mozilla"))
        } else {
            Err(anyhow::anyhow!("not support platform."))
        }
    } else {
        Err(anyhow::anyhow!("not found user dir."))
    }
}
pub fn chrome_path() -> anyhow::Result<PathBuf> {
    if let Some(user_dir) = directories_next::UserDirs::new() {
        let home_dir = user_dir.home_dir();
        if cfg!(target_os = "macos") {
            Ok(home_dir
                .join("Library")
                .join("Application Support")
                .join("Google")
                .join("Chrome")
                .join("NativeMessagingHosts"))
        } else if cfg!(target_os = "linux") {
            Ok(home_dir
                .join(".config")
                .join("google-chrome")
                .join("NativeMessagingHosts"))
        } else if cfg!(target_os = "windows") {
            Ok(home_dir.join("LocalNative").join("config").join("chrome"))
        } else {
            Err(anyhow::anyhow!("not support platform."))
        }
    } else {
        Err(anyhow::anyhow!("not found user dir."))
    }
}
pub fn chromium_path() -> anyhow::Result<PathBuf> {
    if let Some(user_dir) = directories_next::UserDirs::new() {
        let home_dir = user_dir.home_dir();
        if cfg!(target_os = "macos") {
            Ok(home_dir
                .join("Library")
                .join("Application Support")
                .join("Chromium")
                .join("NativeMessagingHosts"))
        } else if cfg!(target_os = "linux") {
            Ok(home_dir
                .join(".config")
                .join("chromium")
                .join("NativeMessagingHosts"))
        } else if cfg!(target_os = "windows") {
            Ok(home_dir.join("LocalNative").join("config").join("chrome"))
        } else {
            Err(anyhow::anyhow!("not support platform."))
        }
    } else {
        Err(anyhow::anyhow!("not found user dir."))
    }
}
pub fn edge_path() -> anyhow::Result<PathBuf> {
    if let Some(user_dir) = directories_next::UserDirs::new() {
        let home_dir = user_dir.home_dir();
        if cfg!(target_os = "macos") {
            // TODO：需要测试
            Ok(home_dir
                .join("Library")
                .join("Application Support")
                .join("Microsoft")
                .join("Edge")
                .join("NativeMessagingHosts"))
        } else if cfg!(target_os = "linux") {
            // TODO:需要测试
            Ok(home_dir
                .join(".config")
                .join("Microsoft")
                .join("Edge")
                .join("NativeMessagingHosts"))
        } else if cfg!(target_os = "windows") {
            Ok(home_dir.join("LocalNative").join("config").join("edge"))
        } else {
            Err(anyhow::anyhow!("not support platform."))
        }
    } else {
        Err(anyhow::anyhow!("not found user dir."))
    }
}
enum WebKind {
    FireFox,
    Chrome,
    Chromium,
    Edge,
}

impl WebKind {
    async fn init_all() -> anyhow::Result<()> {
        if cfg!(windows) {
            if let Some(kind) = registr(Self::FireFox)? {
                try_init_file(kind).await?;
            };
            if let Some(kind) = registr(Self::Chrome)? {
                try_init_file(kind).await?;
            }
            if let Some(kind) = registr(Self::Chromium)? {
                try_init_file(kind).await?;
            }
            if let Some(kind) = registr(Self::Edge)? {
                try_init_file(kind).await?;
            }
            Ok(())
        } else {
            tokio::try_join!(
                try_init_file(Self::FireFox),
                try_init_file(Self::Chrome),
                try_init_file(Self::Chromium),
                try_init_file(Self::Edge)
            )
            .map(|_| ())
        }
    }
    fn registr_path(&self) -> PathBuf {
        match self {
            WebKind::FireFox => Path::new("Software")
                .join("Mozilla")
                .join("NativeMessagingHosts"),
            WebKind::Chrome => Path::new("Software")
                .join("Google")
                .join("Chrome")
                .join("NativeMessagingHosts"),
            WebKind::Chromium => Path::new("Software")
                .join("Chromium")
                .join("NativeMessagingHosts"),
            WebKind::Edge => Path::new("Software")
                .join("Microsoft")
                .join("Edge")
                .join("NativeMessagingHosts"),
        }
    }
    fn path(&self) -> anyhow::Result<PathBuf> {
        match self {
            WebKind::FireFox => firefox_path(),
            WebKind::Chrome => chrome_path(),
            WebKind::Chromium => chromium_path(),
            WebKind::Edge => edge_path(),
        }
    }
    fn host(&self) -> anyhow::Result<AppHost> {
        match self {
            WebKind::FireFox => AppHost::firefox(),
            _ => AppHost::chrome(),
        }
    }
    #[cfg(windows)]
    fn json_path(&self) -> anyhow::Result<String> {
        let path = self.path()?.join("app.localnative.json");
        path.into_os_string()
            .into_string()
            .map_err(|e| anyhow::anyhow!("into string fail{:?}", e))
    }
}
async fn try_init_file(kind: WebKind) -> anyhow::Result<()> {
    log::info!("try_init_file start get kind path.");
    let dir_path = kind.path()?;
    let file_path = dir_path.join("app.localnative.json");
    log::info!("try_init_file start get kind host.");
    let host = kind.host()?;
    log::info!("try_init_file start get raw data.");
    let raw_file = host.raw_data()?;
    if cfg!(windows) {
        init_file(&file_path, &raw_file, &dir_path).await?;
        log::info!("try_init_file init ok.");
    } else if cfg!(unix) {
        if dir_path.exists() {
            init_file(&file_path, &raw_file, &dir_path).await?;
        }
    } else {
        return Err(anyhow::anyhow!("not support platform."));
    }

    Ok(())
}

async fn init_file(
    file_path: &PathBuf,
    raw_file: &Vec<u8>,
    dir_path: &PathBuf,
) -> anyhow::Result<()> {
    Ok(if file_path.exists() {
        log::info!("init_file is reading.");
        let file = tokio::fs::read(file_path).await?;
        log::info!("init_file read fine.");
        if file != *raw_file {
            log::info!("init_file is writing.");
            tokio::fs::write(file_path, raw_file).await?;
            log::info!("init_file write ok.");
        }
    } else {
        log::info!("init_file is creating dir.");
        tokio::fs::create_dir_all(dir_path).await?;
        log::info!("init_file is writing file.");
        tokio::fs::write(file_path, raw_file).await?;
        log::info!("init_file write ok.");
    })
}
