use std::path::Path;

use xshell::{Shell, cmd};

use crate::flags::Release;

impl Release {
    pub fn run(&self) -> anyhow::Result<()> {
        let sh = Shell::new()?;
        let version = self.version.as_deref().unwrap_or(env!("CARGO_PKG_VERSION"));
        println!("Packaging LocalNative CLI {version}");

        cmd!(
            sh,
            "cargo build --release --bin localnative --bin localnative-web-ext-host"
        )
        .run()?;

        let suffix = suffix();
        let src = Path::new("target").join("release");
        let cli_src = src.join(format!("localnative{suffix}"));
        let host_src = src.join(format!("localnative-web-ext-host{suffix}"));
        let dst = std::env::current_dir()?.join("dist");
        if dst.exists() {
            sh.remove_path(&dst)?;
        }
        sh.create_dir(&dst)?;

        sh.copy_file(&cli_src, dst.join(format!("localnative{suffix}")))?;
        sh.copy_file(
            &host_src,
            dst.join(format!("localnative-web-ext-host{suffix}")),
        )?;
        sh.copy_file("README.md", &dst)?;
        sh.copy_file("../LICENSE", &dst)?;
        println!("Wrote {}", dst.display());
        Ok(())
    }
}

fn suffix() -> &'static str {
    if std::env::consts::OS == "windows" {
        ".exe"
    } else {
        ""
    }
}
