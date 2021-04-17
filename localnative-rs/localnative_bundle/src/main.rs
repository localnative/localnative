use std::process::Command;

use tauri_bundler::{
    bundle_project, AppCategory, BundleBinary, BundleSettings, DebianSettings, MacOsSettings,
    PackageSettings, PackageType, Settings, SettingsBuilder, WindowsSettings,
};
struct Bundler {
    debug: bool,
    verbose: bool,
}

impl Bundler {
    pub fn new(debug: bool, verbose: bool) -> Self {
        Self { debug, verbose }
    }
}

fn main() -> anyhow::Result<()> {
    let bundler = Bundler::new(true, true);
    let settings = settings(&bundler)?;
    let mut build_args = vec![
        "build",
        "--package",
        "localnative_iced",
        "--out-dir",
        "./output",
        "-Z",
        "unstable-options",
    ];
    if !bundler.debug {
        build_args.push("--release");
    }
    if !Command::new("cargo").args(&build_args).status()?.success() {
        return Err(anyhow::anyhow!("cargo fail!"));
    }
    bundle_project(settings)?;
    // 方案一：
    // 1. 获取安装目录
    // 2. 由已知安装目录，生成相应的json
    // 3. 将生成的json移动到浏览器插件对应位置
    // 4. 对windows平台做注册表处理
    // 5. 由bundle生成对应的安装文件
    // 6. 需要的binary：neon、iced
    // 7. neon生成的文件放在已知的安装目录

    // 方案二：
    // 1. 设置一个固定的环境变量，指向neon存放的位置，最好是和electron版本放在一个地方
    // 2. 由1中环境变量生成对应json，移动到对应位置，并且对windows平台做注册表处理
    // 3. 由bundler生成对应的安装文件，其中neon文件的位置需要移动到和ekectron一致的位置

    // 方案三：
    // 类似方案二，但是环境变量设置为安装应用程序的目录，其中neon放到用户选择的目录中
    // 并且还要提供一个迁移方案，copy之前用户存放在~/localnatiev中的sqlite3文件，
    // 将这些文件作为最终程序包，这样的好处是易于卸载。
    Ok(())
}

fn settings(bundler: &Bundler) -> anyhow::Result<Settings> {
    let iced_src_path = Some(get_src_path("local_native", bundler));
    let iced_src = BundleBinary::new("localnative_iced".to_owned(), true).set_src_path(iced_src_path);
    let mut seetings_builder = SettingsBuilder::new()
        .binaries(vec![iced_src])
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
                ..Default::default()
            },
        })
        .package_settings(PackageSettings {
            product_name: "Local Native".to_owned(),
            version: "0.4.2".to_owned(),
            description: "localnative iced application".to_owned(),
            homepage: Some("https://localnative.app/".to_owned()),
            authors: None,
            default_run: None,
        })
        .package_types(vec![
            PackageType::WindowsMsi,
            PackageType::MacOsBundle,
            PackageType::Deb,
        ])
        .project_out_directory(r"F:\Documents\GitHub\localnative\localnative-rs\output".to_owned());
    if bundler.verbose {
        seetings_builder = seetings_builder.verbose();
    }
    seetings_builder
        .build()
        .map_err(|err| anyhow::anyhow!("{:?}", err))
}
fn get_src_path(name: &str, bundler: &Bundler) -> String {
    let mut src = "target".to_owned();
    if bundler.debug {
        src += "debug";
    } else {
        src += "release";
    }
    src += name;
    src
}
