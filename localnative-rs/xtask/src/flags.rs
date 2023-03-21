// CICD
// cargo fmt --all -- --check
// cargo clippy --workspace --all-targets --all-features -- -D warnings -A clippy::type_complexity

// Release
// set Cargo.toml version
// cargo build --target {target} --release
// iced_src = target/{target}/release/localnatice_iced + {suffix}
// host_src = target/{target}/release/localnative-web-ext-host + {suffix}
// dst = dist/localnative-{target}-{version}
// cp iced_src dst
// cp host_src dst
// gzip(&src, &dst.with_extension("gz"))?;

xflags::xflags! {
    src "./src/flags.rs"
    cmd xtask {
        /// Release iced and web-ext-host
        cmd release
        {
            /// You can specify a version
            optional -v, --version version: String
        }
        /// Use cargo ndk to build localnative_core to android .so lib
        cmd ndkbd
        {
            /// You can specify a build mode
            optional --debug
        }
    }
}
// generated start
// The following code is generated by `xflags` macro.
// Run `env UPDATE_XFLAGS=1 cargo build` to regenerate.
#[derive(Debug)]
pub struct Xtask {
    pub subcommand: XtaskCmd,
}

#[derive(Debug)]
pub enum XtaskCmd {
    Release(Release),
    Ndkbd(Ndkbd),
}

#[derive(Debug)]
pub struct Release {
    pub version: Option<String>,
}

#[derive(Debug)]
pub struct Ndkbd {
    pub debug: bool,
}

impl Xtask {
    #[allow(dead_code)]
    pub fn from_env_or_exit() -> Self {
        Self::from_env_or_exit_()
    }

    #[allow(dead_code)]
    pub fn from_env() -> xflags::Result<Self> {
        Self::from_env_()
    }

    #[allow(dead_code)]
    pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        Self::from_vec_(args)
    }
}
// generated end
