use std::process::Command;

fn main() {
    let status = Command::new("node")
        .arg("../scripts/build-sidecars.mjs")
        .status()
        .expect("node is required to build the LocalNative sidecars");
    assert!(status.success(), "failed to build LocalNative sidecars");

    for path in [
        "../scripts/build-sidecars.mjs",
        "../../localnative-rs/Cargo.toml",
        "../../localnative-rs/Cargo.lock",
        "../../localnative-rs/localnative_cli/src",
        "../../localnative-rs/localnative_core/src",
        "../../localnative-rs/localnative_hostinstall/src",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    tauri_build::build()
}
