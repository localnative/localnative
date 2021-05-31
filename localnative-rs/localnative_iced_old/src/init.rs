#[cfg(feature = "wgpu")]
use crate::setting_view;
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
        let name;
        #[cfg(target_os = "windows")]
        {
            name = "localnative-web-ext-host.exe";
        }
        #[cfg(not(target_os = "windows"))]
        {
            name = String::from("localnative-web-ext-host");
        }
        let mut path = std::env::current_dir()?;
        path = path.join(name);
        let mut res = path
            .into_os_string()
            .into_string()
            .map_err(|err| anyhow::anyhow!("faid to get path {:?}", err))?;
        res = if cfg!(target_os = "windows") {
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
    WebKind::init_all().await;
    #[cfg(feature = "wgpu")]
    create_env().await?;
    Ok(())
}
pub async fn fix_app_host() {
    WebKind::init_all().await
}
#[cfg(feature = "wgpu")]
pub async fn create_env() -> anyhow::Result<()> {
    let app_dir = setting_view::app_dir();
    if !app_dir.exists() {
        tokio::fs::create_dir_all(&app_dir).await?;
    }
    let env_path = app_dir.join(".env");
    if env_path.is_dir() {
        tokio::fs::remove_dir(&env_path).await?;
    }
    if env_path.exists() && env_path.is_dir() {
        tokio::fs::remove_dir(&env_path).await?;
    }
    tokio::fs::write(env_path, "WGPU_BACKEND=primary").await?;
    Ok(())
}
#[cfg(feature = "wgpu")]
pub async fn change_env(backend: setting_view::Backend) -> anyhow::Result<setting_view::Backend> {
    let app_dir = setting_view::app_dir();
    if !app_dir.exists() {
        tokio::fs::create_dir_all(&app_dir).await?;
    }
    let env_path = app_dir.join(".env");
    if env_path.is_dir() {
        tokio::fs::remove_dir(&env_path).await?;
    }

    if env_path.exists() && env_path.is_dir() {
        tokio::fs::remove_dir(&env_path).await?;
    }
    log::debug!("{} backend will write in env.💥💥💢", backend.to_string());
    let backend_str = match backend {
        setting_view::Backend::Primary => "primary".to_owned(),
        backend => backend.to_string(),
    };
    tokio::fs::write(env_path, format!("WGPU_BACKEND = {}", backend_str))
        .await
        .map(|_| backend)
        .map_err(|e| anyhow::anyhow!("write env fail:{:?}", e))
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
fn registr(kind: WebKind) -> anyhow::Result<bool> {
    use winreg::enums::*;
    log::debug!("registr starting");
    let path = kind.registr_path();
    let write_path = path.join("app.localnative");
    log::debug!("registr write path:{:?}", &write_path);
    log::debug!("registr get kind host");
    let json_path = kind.json_path()?;
    let key = winreg::RegKey::predef(HKEY_CURRENT_USER);

    if let Ok(k) = key.open_subkey(&path) {
        log::debug!("registr open subkey success");
        if let Ok(k) = k.open_subkey(Path::new("app.localnative")) {
            if let Ok(v) = k.get_value::<String, &str>("") {
                log::debug!("registr get value success:{:?}", &v);
                if v != json_path {
                    log::debug!("registr start set value");
                    let writer = key.open_subkey_with_flags(&write_path, KEY_WRITE)?;
                    writer.set_value("", &json_path)?;
                    log::debug!("registr set value ok");
                } else {
                    log::debug!("registr value Eq");
                }
                return Ok(true);
            } else {
                log::error!("get registr value fail!");
            }
        } else {
            log::debug!("registr start set value");
            let (writer, _disposition) = key.create_subkey_with_flags(&write_path, KEY_WRITE)?;
            writer.set_value("", &json_path)?;
            log::debug!("registr set value ok");
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn firefox_path() -> anyhow::Result<PathBuf> {
    if let Some(user_dir) = directories_next::UserDirs::new() {
        let home_dir = user_dir.home_dir();
        #[cfg(target_os = "macos")]
        {
            Ok(home_dir
                .join("Library")
                .join("Application Support")
                .join("Mozilla"))
        }
        #[cfg(target_os = "linux")]
        {
            Ok(home_dir.join(".mozilla"))
        }
        #[cfg(target_os = "windows")]
        {
            Ok(home_dir.join("LocalNative").join("config").join("mozilla"))
        }
    } else {
        Err(anyhow::anyhow!("not found user dir."))
    }
}
pub fn chrome_path() -> anyhow::Result<PathBuf> {
    if let Some(user_dir) = directories_next::UserDirs::new() {
        let home_dir = user_dir.home_dir();
        #[cfg(target_os = "macos")]
        {
            Ok(home_dir
                .join("Library")
                .join("Application Support")
                .join("Google")
                .join("Chrome"))
        }
        #[cfg(target_os = "linux")]
        {
            Ok(home_dir.join(".config").join("google-chrome"))
        }
        #[cfg(target_os = "windows")]
        {
            Ok(home_dir.join("LocalNative").join("config").join("chrome"))
        }
    } else {
        Err(anyhow::anyhow!("not found user dir."))
    }
}
pub fn chromium_path() -> anyhow::Result<PathBuf> {
    if let Some(user_dir) = directories_next::UserDirs::new() {
        let home_dir = user_dir.home_dir();
        #[cfg(target_os = "macos")]
        {
            Ok(home_dir
                .join("Library")
                .join("Application Support")
                .join("Chromium"))
        }
        #[cfg(target_os = "linux")]
        {
            Ok(home_dir.join(".config").join("chromium"))
        }
        #[cfg(target_os = "windows")]
        {
            Ok(home_dir.join("LocalNative").join("config").join("chrome"))
        }
    } else {
        Err(anyhow::anyhow!("not found user dir."))
    }
}
pub fn edge_path() -> anyhow::Result<PathBuf> {
    if let Some(user_dir) = directories_next::UserDirs::new() {
        let home_dir = user_dir.home_dir();
        #[cfg(target_os = "macos")]
        {
            // TODO：需要测试
            Ok(home_dir
                .join("Library")
                .join("Application Support")
                .join("Microsoft")
                .join("Edge"))
        }
        #[cfg(target_os = "linux")]
        {
            // TODO:需要测试
            Ok(home_dir.join(".config").join("Microsoft").join("Edge"))
        }
        #[cfg(target_os = "windows")]
        {
            Ok(home_dir.join("LocalNative").join("config").join("edge"))
        }
    } else {
        Err(anyhow::anyhow!("not found user dir."))
    }
}
#[derive(Debug, Clone, Copy)]
enum WebKind {
    FireFox,
    Chrome,
    Chromium,
    Edge,
}

impl WebKind {
    async fn init_all() {
        tokio::join!(
            try_init_file(Self::FireFox),
            try_init_file(Self::Chrome),
            try_init_file(Self::Chromium),
            try_init_file(Self::Edge)
        );
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
        let browser_path = self.browser_path()?;
        #[cfg(not(target_os = "windows"))]
        if browser_path.exists() {
            #[cfg(target_os = "macos")]
            {
                Ok(browser_path.join("NativeMessagingHosts"))
            }
            #[cfg(target_os = "linux")]
            match self {
                WebKind::FireFox => Ok(browser_path.join("native-messaging-hosts")),
                _ => Ok(browser_path.join("NativeMessagingHosts")),
            }
            #[cfg(target_os = "windows")]
            Ok(browser_path)
        } else {
            Err(anyhow::anyhow!("not exists the browser:{:?}", self))
        }
        #[cfg(target_os = "windows")]
        Ok(browser_path)
    }
    fn browser_path(&self) -> anyhow::Result<PathBuf> {
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
    #[cfg(target_os = "windows")]
    fn json_path(&self) -> anyhow::Result<String> {
        let path = self.path()?.join("app.localnative.json");
        path.into_os_string()
            .into_string()
            .map_err(|e| anyhow::anyhow!("into string fail{:?}", e))
    }
}
async fn try_init_file(kind: WebKind) {
    log::info!("Start try init file");
    #[cfg(target_os = "windows")]
    {
        match registr(kind) {
            Ok(false) => {
                log::info!("this kind browser is not exists :{:?}", kind);
                return;
            }
            Err(e) => {
                log::error!("registr error:{:?}", e);
                return;
            }
            _ => {}
        }
    }

    let dir_path = match kind.path() {
        Ok(path) => path,
        Err(e) => {
            log::warn!("get dir path error:{:?}", e);
            return;
        }
    };
    let file_path = dir_path.join("app.localnative.json");
    log::debug!("try_init_file start get kind host.");
    let host = match kind.host() {
        Ok(host) => host,
        Err(e) => {
            log::warn!("get host error:{:?}", e);
            return;
        }
    };
    log::debug!("try_init_file start get raw data.");
    let raw_file = match host.raw_data() {
        Ok(file) => file,
        Err(e) => {
            log::warn!("get raw file error:{:?}", e);
            return;
        }
    };
    #[cfg(target_os = "windows")]
    {
        match init_file(&file_path, &raw_file, &dir_path).await {
            Ok(_) => {
                let host_path = Path::new(&host.path);
                if !host_path.exists() {
                    log::error!(
                        "try init file fail, web ext host is not exists {:?}",
                        host_path
                    );
                } else {
                    log::info!("try init file fine.");
                }
            }
            Err(e) => {
                log::error!("init host file error:{:?}", e);
            }
        };
    }
    #[cfg(not(target_os = "windows"))]
    {
        match init_file(&file_path, &raw_file, &dir_path).await {
            Ok(_) => {
                let host_path = Path::new(&host.path);
                if !host_path.exists() {
                    log::error!(
                        "try init file fail, web ext host is not exists {:?}",
                        host_path
                    );
                } else {
                    log::info!("try init file fine.");
                }
            }
            Err(e) => {
                log::error!("init host file error:{:?}", e);
            }
        };
    }
}

async fn init_file(file_path: &Path, raw_file: &[u8], dir_path: &Path) -> anyhow::Result<()> {
    if file_path.exists() {
        log::debug!("init_file is reading.");
        let file = tokio::fs::read(file_path).await?;
        log::debug!("init_file read fine.");
        if file != *raw_file {
            log::debug!("init_file is writing.");
            tokio::fs::write(file_path, raw_file).await?;
            log::debug!("init_file write ok.");
        }
    } else {
        log::debug!("init_file is creating dir.");
        tokio::fs::create_dir_all(dir_path).await?;
        log::debug!("init_file is writing file.");
        tokio::fs::write(file_path, raw_file).await?;
        log::debug!("init_file write ok.");
    }
    Ok(())
}
