pub mod sync;
extern crate serde_json;
use std::process::Command;
use SsbNote;

pub fn sync_to_db() {}

pub fn whoami() -> String {
    let output = Command::new("node")
        .arg("../../localnative-nodejs/whoami.js")
        .output()
        .expect("failed to execute process");

    // eprintln!("status: {}", output.status);
    // eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout)
        .trim_right()
        .to_string()
}

pub fn tail(id: &str, gt: i64) -> Option<SsbNote> {
    let output = Command::new("node")
        .arg("../../localnative-nodejs/tail.js")
        .arg(id)
        .arg(gt.to_string())
        .output()
        .expect("failed to execute process");

    // eprintln!("status: {}", output.status);
    // eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());

    let text = String::from_utf8_lossy(&output.stdout);

    if let Ok(i) = serde_json::from_str::<SsbNote>(&text) {
        Some(i)
    } else {
        None
    }
}

pub fn publish() -> String {
    let output = Command::new("node")
        .arg("../../localnative-nodejs/publish.js")
        .output()
        .expect("failed to execute process");

    // eprintln!("status: {}", output.status);
    // eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout)
        .trim_right()
        .to_string()
}
