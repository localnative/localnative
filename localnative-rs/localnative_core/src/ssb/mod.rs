pub mod sync;
extern crate rusqlite;
extern crate serde_json;
use rusqlite::Connection;

use std::process::Command;
extern crate dirs;
use Note;
use SsbNote;

fn node_dir() -> String {
    let p = format!(
        "{}/.localnative/localnative/localnative-nodejs",
        dirs::home_dir().unwrap().to_str().unwrap()
    );
    // println!("{}", p);
    p
}

fn node_exe() -> String {
    "node".to_string()
}

pub fn run_sync(conn: &Connection) {
    let id = whoami();
    sync::init_active_author(&conn, &id);
    sync::sync_to_ssb(&conn);
    sync::sync_to_db(&conn, &id);
}

pub fn whoami() -> String {
    // let output = Command::new(node_exe())
    let output = Command::new(node_exe())
        .arg(format!("{}/whoami.js", node_dir()))
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
    let output = Command::new(node_exe())
        .arg(format!("{}/tail.js", node_dir()))
        .arg(id)
        .arg(gt.to_string())
        .output()
        .expect("failed to execute process");

    eprintln!("status: {}", output.status);
    eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());

    let text = String::from_utf8_lossy(&output.stdout);

    match serde_json::from_str::<SsbNote>(&text) {
        Ok(i) => Some(i),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn publish(note: &Note) -> SsbNote {
    let note_json = json!(note).to_string();

    // eprintln!("{}", note_json);

    let output = Command::new(node_exe())
        .arg(format!("{}/publish.js", node_dir()))
        .arg(note_json)
        .output()
        .expect("failed to execute process");

    // eprintln!("status: {}", output.status);
    // eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    serde_json::from_str::<SsbNote>(&text).unwrap()
}
