use std::process::Command;

use serde::Deserialize;
use tauri_bundler::{
    bundle_project, AppCategory, BundleBinary, BundleSettings, DebianSettings, MacOsSettings,
    PackageSettings, PackageType, Settings, SettingsBuilder, WindowsSettings,
};

#[derive(Debug, Deserialize)]
struct Bundler {
    verbose: bool,
}

impl Bundler {
    pub fn new() -> anyhow::Result<Self> {
        let file = include_str!("../../bundler.json");
        serde_json::from_str(file).map_err(|e| anyhow::anyhow!("{}", e))
    }
}

fn main() -> anyhow::Result<()> {
    let bundler = Bundler::new()?;
    let settings = settings(&bundler)?;
    build_iced()?;
    build_web_ext_host()?;
    bundle_project(settings)?;
    Ok(())
}

fn settings(bundler: &Bundler) -> anyhow::Result<Settings> {
    let iced_src_path = Some(get_src_path("localnative_iced"));
    let iced_src =
        BundleBinary::new("localnative_iced".to_owned(), true).set_src_path(iced_src_path);
    let host_src_path = Some(get_src_path("localnative-web-ext-host"));
    let host_src =
        BundleBinary::new("localnative-web-ext-host".to_owned(), false).set_src_path(host_src_path);
    let mut seetings_builder = SettingsBuilder::new()
        .binaries(vec![iced_src, host_src])
        .bundle_settings(BundleSettings {
            identifier: Some("com.localnative.iced".to_owned()),
            icon: Some(vec![
                "/icons/512x512.png".to_owned(),
                "/icons/icon.icns".to_owned(),
                "/icons/icon.ico".to_owned(),
            ]),
            resources: None,
            copyright: None,
            category: Some(AppCategory::Utility),
            short_description: Some("Local Native".to_owned()),
            long_description: Some("Local Native, a utilit application.".to_owned()),
            bin: None,
            external_bin: None,
            deb: DebianSettings::default(),
            macos: MacOsSettings::default(),
            updater: None,
            windows: WindowsSettings {
                template: Some("./templates/main.wxs".to_owned()),
                ..Default::default()
            },
        })
        .package_settings(PackageSettings {
            product_name: "Local Native".to_owned(),
            version: "0.4.2".to_owned(),
            description: "localnative iced application".to_owned(),
            homepage: Some("https://localnative.app/".to_owned()),
            authors: Some(vec!["Cupnfish".to_owned()]),
            default_run: Some("localnative_iced".to_owned()),
        })
        .package_types(vec![
            PackageType::WindowsMsi,
            PackageType::MacOsBundle,
            PackageType::Deb,
            PackageType::AppImage,
        ])
        .project_out_directory(project_out_dir()?);
    if bundler.verbose {
        seetings_builder = seetings_builder.verbose();
    }
    seetings_builder
        .build()
        .map_err(|err| anyhow::anyhow!("{:?}", err))
}
fn get_src_path(name: &str) -> String {
    let mut src = "target".to_owned();
    if cfg!(debug_assertions) {
        src += "debug";
    } else {
        src += "release";
    }
    src += name;
    src
}
fn project_out_dir() -> anyhow::Result<String> {
    let mut dir = std::env::current_dir()?;
    dir = dir.join("output");
    let mut res = dir
        .into_os_string()
        .into_string()
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    res = if cfg!(windows) {
        res.replace("//", "/")
    } else {
        res
    };
    Ok(res)
}
fn build_iced() -> anyhow::Result<()> {
    let mut build_args = vec![
        "build",
        "--package",
        "localnative_iced",
        "--out-dir",
        "./output",
        "-Z",
        "unstable-options",
    ];
    if !cfg!(debug_assertions) {
        build_args.push("--release");
    }
    if !Command::new("cargo").args(&build_args).status()?.success() {
        return Err(anyhow::anyhow!("build iced fail!"));
    }
    Ok(())
}
fn build_web_ext_host() -> anyhow::Result<()> {
    let mut build_args = vec![
        "build",
        "--package",
        "localnative_cli",
        "--bin",
        "localnative-web-ext-host",
        "--out-dir",
        "./output",
        "-Z",
        "unstable-options",
    ];
    if !cfg!(debug_assertions) {
        build_args.push("--release");
    }
    if !Command::new("cargo").args(&build_args).status()?.success() {
        return Err(anyhow::anyhow!("build web host fail!"));
    }
    Ok(())
}
