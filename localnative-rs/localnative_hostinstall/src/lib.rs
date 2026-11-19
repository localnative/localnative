/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! One installer for the browser-extension native-messaging manifests, shared
//! by the desktop front-ends and the `localnative host install` command.
//!
//! Each browser discovers the permanent `localnative-web-ext-host` alias through
//! a JSON manifest at a platform- and browser-specific location. This crate
//! computes those locations, writes (or refreshes) the manifests, and installs
//! the host executable where the manifests can find it.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// The manifest one browser reads to find the extension host binary.
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
    /// Where the host binary is expected to live for the manifest to point at.
    ///
    /// macOS/Windows: next to the running app or CLI binary.
    /// Linux: `~/LocalNative/localnative-web-ext-host`. Desktop installers copy
    /// the sibling host alias; the CLI installer copies its unified executable.
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
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        #[cfg(target_os = "linux")]
        let mut path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("LocalNative");

        path = path.join(name);
        tracing::debug!(?path, "web-ext-host path");
        path
    }

    pub fn new(
        name: String,
        description: String,
        path: PathBuf,
        tp: String,
        allowed_extensions: Option<Vec<String>>,
        allowed_origins: Option<Vec<String>>,
    ) -> Self {
        Self {
            name,
            description,
            path,
            tp,
            allowed_extensions,
            allowed_origins,
        }
    }

    pub fn firefox() -> Self {
        Self::new(
            "app.localnative".to_owned(),
            "Local Native Host".to_owned(),
            Self::path(),
            "stdio".to_owned(),
            Some(vec!["localnative@example.org".to_owned()]),
            None,
        )
    }

    pub fn chrome(allowed_origins: Option<Vec<String>>) -> Self {
        Self::new(
            "app.localnative".to_owned(),
            "Local Native Host".to_owned(),
            Self::path(),
            "stdio".to_owned(),
            None,
            allowed_origins.or_else(|| {
                Some(vec![
                    "chrome-extension://oclkmkeameccmgnajgogjlhdjeaconnb/".to_owned(),
                ])
            }),
        )
    }

    pub fn raw_data(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

#[cfg(target_os = "windows")]
fn registr(kind: WebKind) -> std::io::Result<()> {
    use winreg::enums::*;

    let path = kind.registr_path();
    let write_path = path.join("app.localnative");
    let json_path = kind.json_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "manifest path is not valid Unicode",
        )
    })?;
    let key = winreg::RegKey::predef(HKEY_CURRENT_USER);
    let value = key
        .open_subkey(&path)
        .and_then(|k| k.open_subkey(Path::new("app.localnative")))
        .and_then(|k| k.get_value::<String, &str>(""))
        .ok();
    let write = || {
        key.open_subkey_with_flags(&write_path, KEY_WRITE)
            .and_then(|writer| writer.set_value("", &json_path))
            .or_else(|_| {
                key.create_subkey_with_flags(&write_path, KEY_WRITE)
                    .map(|(writer, _)| writer.set_value("", &json_path))
                    .and_then(std::convert::identity)
            })
    };
    match value {
        Some(value) if value == json_path => Ok(()),
        _ => write(),
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
    /// Install both manifests (and, on Linux, the host binary copy), logging
    /// any failure for compatibility with existing desktop callers.
    pub fn install_all(allowed_origins: Option<Vec<String>>) {
        if let Err(error) = Self::try_install_all(allowed_origins) {
            tracing::error!(%error, "failed to install browser native messaging");
        }
    }

    /// Fallible desktop installation used to report the result in a UI.
    /// `allowed_origins` overrides the Chrome extension ID list (used when
    /// loading the extension unpacked during development).
    pub fn try_install_all(allowed_origins: Option<Vec<String>>) -> std::io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let exe = std::env::current_exe()?;
            let home = dirs::home_dir()
                .ok_or_else(|| std::io::Error::other("could not determine home directory"))?;
            let parent = exe
                .parent()
                .ok_or_else(|| std::io::Error::other("executable has no parent directory"))?;
            let from = parent.join("localnative-web-ext-host");
            let to = home.join("LocalNative").join("localnative-web-ext-host");
            copy_host(&from, &to)?;
        }
        try_init_file(Self::FireFox, None)?;
        try_init_file(Self::Chrome, allowed_origins)
    }

    /// Install the currently running unified CLI as the permanent host alias,
    /// then write both browser manifests. Every failure is returned so the
    /// command can set a failing exit status.
    pub fn install_cli(allowed_origins: Option<Vec<String>>) -> std::io::Result<PathBuf> {
        let source = std::env::current_exe()?;
        let destination = AppHost::path();
        copy_host(&source, &destination)?;
        try_init_file(Self::FireFox, None)?;
        try_init_file(Self::Chrome, allowed_origins)?;
        Ok(destination)
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

    // On Windows the map body returns browser_path unchanged; other platforms transform it.
    #[allow(clippy::map_identity)]
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

    fn host(&self, allowed_origins: Option<Vec<String>>) -> AppHost {
        match self {
            WebKind::FireFox => AppHost::firefox(),
            WebKind::Chrome => AppHost::chrome(allowed_origins),
        }
    }

    #[cfg(target_os = "windows")]
    fn json_path(&self) -> Option<String> {
        let path = self.path()?.join("app.localnative.json");
        path.into_os_string().into_string().ok()
    }
}

fn copy_host(source: &Path, destination: &Path) -> std::io::Result<()> {
    if let (Ok(source_path), Ok(destination_path)) = (
        std::fs::canonicalize(source),
        std::fs::canonicalize(destination),
    ) && source_path == destination_path
    {
        return Ok(());
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if destination.is_dir() {
        std::fs::remove_dir(destination)?;
    }
    std::fs::copy(source, destination)?;
    Ok(())
}

fn try_init_file(kind: WebKind, allowed_origins: Option<Vec<String>>) -> std::io::Result<()> {
    let Some(dir_path) = kind.path() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine the browser manifest directory",
        ));
    };
    let raw_file = kind.host(allowed_origins).raw_data();
    init_file(&dir_path, &raw_file)?;
    #[cfg(target_os = "windows")]
    registr(kind)?;
    Ok(())
}

fn init_file(dir_path: &Path, raw_file: &[u8]) -> std::io::Result<()> {
    let file_path = dir_path.join("app.localnative.json");
    if file_path.exists() {
        if file_path.is_file() {
            let file = std::fs::read(&file_path)?;
            if file != *raw_file {
                std::fs::write(&file_path, raw_file)?;
            }
        } else {
            std::fs::remove_dir(&file_path)?;
            create_and_write_file(dir_path, &file_path, raw_file)?;
        }
    } else {
        create_and_write_file(dir_path, &file_path, raw_file)?;
    }

    Ok(())
}

fn create_and_write_file(
    dir_path: &Path,
    file_path: &Path,
    raw_file: &[u8],
) -> std::io::Result<()> {
    std::fs::create_dir_all(dir_path)?;
    std::fs::write(file_path, raw_file)?;
    Ok(())
}
