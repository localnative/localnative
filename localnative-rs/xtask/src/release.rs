use std::{
    fs,
    path::Path,
};

use tauri_bundler::{
    AppCategory, BundleBinary, BundleSettings, DebianSettings, MacOsSettings, PackageSettings,
    PackageType, SettingsBuilder, WindowsSettings,
};
use xshell::{cmd, cp, mkdir_p, rm_rf};

use crate::flags::Release;
// Release
// set Cargo.toml version
// cargo build --target {target} --release
// iced_src = target/{target}/release/localnatice_iced + {suffix}
// host_src = target/{target}/release/localnative-web-ext-host + {suffix}
// dst = dist/localnative-{target}-{version}
// cp iced_src dst
// cp host_src dst
// gzip(&src, &dst.with_extension("gz"))?;
impl Release {
    pub fn run(&self) -> anyhow::Result<()> {
        let version = if let Some(ref version) = self.version {
            version.as_str()
        } else {
            env!("CARGO_PKG_VERSION")
        };
        cmd!("cargo build --no-default-features --features wgpu --release").run()?;
        let suffix = suffix();
        let src = Path::new("target").join("release");
        let iced_src = src.join(format!("local-native{}", suffix));
        let core_rlib = src.join("liblocalnative_core.rlib");
        let iced_rlib = src.join("liblocalnative_iced.rlib");
        let host_src = src.join(format!("localnative-web-ext-host{}", suffix));

        // let tag = format!(
        //     "{}-{}-{}",
        //     std::env::consts::ARCH,
        //     std::env::consts::OS,
        //     version
        // );
        let dst = Path::new("dist");
        if dst.exists() {
            rm_rf(&dst)?;
        }
        mkdir_p(&dst)?;
        let iced_dst = dst.join(format!("local-native{}", suffix));
        let host_dst = dst.join(format!("localnative-web-ext-host{}", suffix));
        let iced_rlib_dst = dst.join("liblocalnative_iced.rlib");
        let core_rlib_dst = dst.join("liblocalnative_core.rlib");
        let readme = Path::new("README.md");
        cp(&readme, &dst)?;
        cp(&iced_src, &iced_dst)?;
        cp(&host_src, &host_dst)?;
        cp(&core_rlib, &core_rlib_dst)?;
        cp(&iced_rlib, &iced_rlib_dst)?;
        copy_dir_all(
            &Path::new("../localnative-electron/build"),
            &dst.join("build"),
        )?;
        let mut package_types = vec![];
        #[cfg(target_os = "macos")]
        package_types.push(PackageType::MacOsBundle);
        #[cfg(target_os = "linux")]
        package_types.push(PackageType::MacOsBundle);
        #[cfg(target_os = "windows")]
        package_types.push(PackageType::MacOsBundle);
        //gzip(&dst, &dst.with_extension("gz"))?;
        let settings = SettingsBuilder::new()
            .verbose()
            .package_settings(PackageSettings {
                product_name: "Local Native".into(),
                version: version.into(),
                description: "Local Native is a open source application...".into(),
                homepage: Some("https://localnative.app/".into()),
                authors: Some(vec![
                    "Yi Wang".into(),
                    "Cupnfish".into(),
                    "Hill Chen".into(),
                ]),
                default_run: Some("local-native".into()),
            })
            .project_out_directory(dst)
            .bundle_settings(BundleSettings {
                identifier: Some("app.localnative".into()),
                icon: Some(vec![
                    "icons/icon.ico".into(),
                    "icons/app.icns".into(),
                    "icons/512x512.png".into(),
                    "icons/1024x1024.png".into(),
                ]),
                resources: None,
                copyright: Some("GNU Affero General Public License v3.0".into()),
                category: Some(AppCategory::Utility),
                short_description: Some("Local Native notes taking.".into()),
                long_description: Some(
                    "localnative iced application, a open source application to manage your notes."
                        .into(),
                ),
                bin: None,
                external_bin: None,
                deb: DebianSettings {
                    ..Default::default()
                },
                macos: MacOsSettings {
                    ..Default::default()
                },
                updater: None,
                windows: WindowsSettings {
                    ..Default::default()
                },
            })
            .binaries(vec![
                BundleBinary::new("local-native".into(), true),
                BundleBinary::new("localnative-web-ext-host".into(), false),
                BundleBinary::new("liblocalnative_core.rlib".into(), false),
                BundleBinary::new("liblocalnative_iced.rlib".into(), false),
            ])
            .package_types(package_types)
            .build()?;
        tauri_bundler::bundle_project(settings)?;
        Ok(())
    }
}

fn suffix() -> String {
    if std::env::consts::OS == "windows" {
        ".exe".into()
    } else {
        "".into()
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
