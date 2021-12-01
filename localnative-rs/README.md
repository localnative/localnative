# localnative-rs

`cargo build`

# localnative-iced

### windows and mac

`cargo build`

### linux

Dependencies:
```bash
sudo apt install pkg-config libfreetype6-dev libfontconfig1-dev libxkbcommon-dev
```
and then `cargo build`

### Release
Our project uses [Tauri Bundler](https://github.com/tauri-apps/tauri/tree/dev/tooling/bundler) for packaging of various platforms. Thank you very much for the Tauri team’s contribution to the Rust community.

You can use the following command to package the program:
```bash
cargo xtask release
```

help:
```bash
xtask release
  Release local native

  OPTIONS:
    -v, --version <version>
      You can specify a version
```

If you do not set the version number, output the version according to the version of the xtask package.

At present, the dmg format of the mac platform will be a bit problematic on my computer, so I commented it out in the `package_types` of `localnative-rs/xtask/src/release.rs`, if you want to try whether it can be packaged normally on your computer, It can be restored. In the current packager, the `mis` package language under windows is set to Chinese, you can change it according to your own language, and then package it later.
### cargo xtask ndkbd

You can use `cargo xtask ndkbd` to compile `localnative_core` as *.so files, and these files will be copied to the corresponding folder. Before you are ready to execute this command, you need to make sure that the [`cargo-ndk`](https://lib.rs/crates/cargo-ndk) tool is installed on your computer. You can use `cargo install cargo-ndk` to download and install. In addition, you need to make sure that valid Android development is installed on your computer. The environment includes SDK, NDK, and the two environment variables `ANDROID_SDK_ROOT` and `NDK_HOME`. 


## Build Android library on macOS

- [Download Android NDK r22b(22.1.7171670)](download ndk: https://developer.android.com/ndk/downloads)
- allow clang etc in Security & Privacy while running for the first time
```shell
cargo xtask ndkbd
```
https://www.lifewire.com/fix-developer-cannot-be-verified-error-5183898


## License
[AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html)

