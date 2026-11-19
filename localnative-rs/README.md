# localnative-rs

`cargo build`

# localnative CLI

The unified `localnative` command provides search, retrieval, import/export,
sync, browser-host installation, and a read-only MCP server. Run
`cargo run -p localnative_cli -- --help` for all subcommands.

The MCP server exposes `search_notes` and `get_note` over stdio:

```bash
cargo build --release --bin localnative --bin localnative-web-ext-host
./target/release/localnative mcp
```

Release bundles include the unified CLI and permanent browser-host alias. The
former standalone commands remain compatibility aliases in this release. Client
configuration and troubleshooting are documented at
https://localnative.app/docs/mcp.

### CLI release

```bash
cargo xtask release
```

This writes `localnative`, `localnative-web-ext-host`, the README and LICENSE to
`dist/`. The optional `-v <version>` overrides the displayed package version.
Desktop installers are built separately from `localnative-tauri` with
`cargo tauri build` or its GitHub Actions workflow.

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

