use xshell::{Shell, cmd};

use crate::flags::Ndkbd;

impl Ndkbd {
    /// Build the Android `liblocalnative_core.so` for every ABI straight into
    /// `localnative-android/app/src/main/jniLibs/` (cargo-ndk `-o`).
    pub fn run(&self) -> anyhow::Result<()> {
        let sh = Shell::new()?;

        let _p = sh.push_dir("localnative_core");
        let out = "../../localnative-android/app/src/main/jniLibs";

        if self.debug {
            cmd!(
                sh,
                "cargo ndk -t armeabi-v7a -t arm64-v8a -t x86 -t x86_64 -o {out} build"
            )
        } else {
            cmd!(
                sh,
                "cargo ndk -t armeabi-v7a -t arm64-v8a -t x86 -t x86_64 -o {out} build --release"
            )
        }
        .run()?;

        Ok(())
    }
}
