use std::fs;
use std::fs::File;
use std::io::{Read, Write};

use std::path::Path;

use xshell::{cmd, cp, mkdir_p, read_dir, rm_rf};
use zip::write::FileOptions;

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
        let features = if self.opengl { "opengl" } else { "wgpu" };
        let target = if let Some(ref platform) = self.platform {
            match platform.as_str() {
                "linux" => "x86_64-unknown-linux-gnu",
                "macos" => "x86_64-apple-darwin",
                "windows" => "x86_64-pc-windows-msvc",
                _ => self.get_target(),
            }
        } else {
            self.get_target()
        };
        let version = if let Some(ref version) = self.version {
            version.as_str()
        } else {
            env!("CARGO_PKG_VERSION")
        };
        cmd!("cargo build --target {target} --features {features} --no-default-features --release")
            .run()?;
        let suffix = exe_suffix(&target);
        let src = Path::new("target").join(&target).join("release");
        let iced_src = src.join(format!("localnative_iced{}", suffix));
        let host_src = src.join(format!("localnative-web-ext-host{}", suffix));
        let dst =
            Path::new("dist").join(format!("localnative-{}-{}-{}", target, features, version));
        rm_rf(&dst)?;
        mkdir_p(&dst)?;
        let iced_dst = dst.join(format!("localnative{}", suffix));
        let host_dst = dst.join(format!("localnative-web-ext-host{}", suffix));
        let core_src = src.join("liblocalnative_core.rlib");
        let core_dst = dst.join("liblocalnative_core.rlib");
        let readme = Path::new("README.md");
        cp(&readme, &dst)?;
        cp(&iced_src, &iced_dst)?;
        cp(&host_src, &host_dst)?;
        cp(&core_src, &core_dst)?;
        copy_dir_all(&Path::new("locales"), &dst.join("locales"))?;
        gzip(&dst, &dst.with_extension("gz"))?;
        Ok(())
    }

    fn get_target(&self) -> &str {
        match self.target.as_ref() {
            Some(target) => target,
            None => {
                if cfg!(target_os = "linux") {
                    "x86_64-unknown-linux-gnu"
                } else if cfg!(target_os = "windows") {
                    "x86_64-pc-windows-msvc"
                } else if cfg!(target_os = "macos") {
                    "x86_64-apple-darwin"
                } else {
                    panic!("Unsupported OS, maybe try setting target")
                }
            }
        }
    }
}
fn gzip(src_path: &Path, dst_path: &Path) -> anyhow::Result<()> {
    rm_rf(&dst_path)?;
    let file = File::create(&dst_path)?;
    let mut encoder = zip::ZipWriter::new(file);
    let options = FileOptions::default();
    let paths = read_dir(src_path)?;
    compress(paths, &mut encoder, options, src_path)?;
    encoder.finish()?;
    Ok(())
}

fn compress(
    paths: Vec<std::path::PathBuf>,
    encoder: &mut zip::ZipWriter<File>,
    options: FileOptions,
    src_path: &Path,
) -> Result<(), anyhow::Error> {
    let mut buffer = vec![];
    for path in paths {
        println!("Compressing {:?}", &path);
        let name = path.strip_prefix(src_path)?.to_str().unwrap();
        if path.is_file() {
            encoder.start_file(name, options)?;
            let mut file = File::open(&path)?;
            file.read_to_end(&mut buffer)?;
            encoder.write_all(&*buffer)?;
            buffer.clear();
        } else if !name.is_empty() {
            encoder.add_directory(name, options)?;
            compress(read_dir(path)?, encoder, options, src_path)?;
        }
    }
    Ok(())
}
fn exe_suffix(target: &str) -> String {
    if target.contains("-windows-") {
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
