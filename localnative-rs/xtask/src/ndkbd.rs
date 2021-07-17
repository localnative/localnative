use std::{path::PathBuf, str::FromStr};

use xshell::{cmd, cp, pushd, rm_rf};

use crate::flags::Ndkbd;

impl Ndkbd {
    pub fn run(&self) -> anyhow::Result<()> {
        let _p = pushd("localnative_core")?;

        if self.debug {
            cmd!("cargo ndk -t armeabi-v7a -t arm64-v8a -t x86 -t x86_64 build")
        } else {
            cmd!("cargo ndk -t armeabi-v7a -t arm64-v8a -t x86 -t x86_64 build --release")
        }
        .run()?;

        let _p = pushd("..")?;
        // let cwd = cwd()?;
        // let to = cwd
        //     .parent()
        //     .unwrap()
        //     .join("localnative-android")
        //     .join("app")
        //     .join("src")
        //     .join("main")
        //     .join("jniLibs");
        let to = PathBuf::from_str("../localnative-android/app/src/main/jniLibs")?;
        //let from = cwd.join("target");
        let from = PathBuf::from("./target");

        let mode = if self.debug { "debug" } else { "release" };

        let name = "liblocalnative_core.so";
        let to_paths = vec![
            to.join("armeabi-v7a"),
            to.join("arm64-v8a"),
            to.join("x86"),
            to.join("x86_64")
        ];
        let from_paths = vec![
            from.join("armv7-linux-androideabi"),
            from.join("aarch64-linux-android"),
            from.join("i686-linux-android"),
            from.join("x86_64-linux-android")
        ];

        for (to,from) in to_paths.iter().zip(from_paths.iter()) {
            let from = from.join(mode).join(name);
            let to = to.join(name);
            if to.exists() {
                rm_rf(&to)?;
            }
            cp(from, to)?;
        }

        Ok(())
    }
}
