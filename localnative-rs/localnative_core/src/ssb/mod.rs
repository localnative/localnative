pub mod sync;
extern crate rusqlite;
extern crate serde_json;
use rusqlite::Connection;
use std::path::Path;
use std::process::{Command, Stdio};
extern crate dirs;
use std::fs;
use std::io::Write;
use Note;
use SsbNote;
use Ssbify;

pub fn get_sqlite_connection() -> Connection {
    let p = sqlite3_db_location();
    let path = Path::new(&p);
    let conn = Connection::open(path).unwrap();
    conn
}

fn sqlite3_db_location() -> String {
    if cfg!(target_os = "android") {
        return "/sdcard/localnative.sqlite3".to_string();
    }
    let mut dir_name = ".ssb"; // for desktop to co-locate with .ssb
    if cfg!(target_os = "ios") {
        dir_name = "Documents";
    }
    let dir = format!(
        "{}/{}",
        dirs::home_dir().unwrap().to_str().unwrap(),
        dir_name
    );
    eprintln!("db dir location: {}", dir);
    fs::create_dir_all(&dir).unwrap();
    format!("{}/localnative.sqlite3", dir)
}

pub fn run_sync() {
    let id = whoami();
    let conn = get_sqlite_connection();
    sync::init_active_author(&conn, &id);
    sync::sync_to_ssb(&conn);
    sync::sync_all_to_db();
}

pub fn ssbify(content: &str, title: &str, url: &str) -> Option<Ssbify> {
    let mut child = Command::new("localnative-ssbify")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("-")
        .arg(title)
        .arg(url)
        .spawn()
        .expect("failed to execute ssbify");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(content.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    eprintln!("status: {}", output.status);
    eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);

    match serde_json::from_str::<Ssbify>(&text) {
        Ok(i) => Some(i),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn whoami() -> String {
    let output = Command::new("localnative-ssb-whoami")
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
    let output = Command::new("localnative-ssb-tail")
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

pub fn publish(note: Note, pubkeys: &str) -> SsbNote {
    let size = 4000;
    if &note.annotations.trim().to_string() == "" {
        let ssb_note = publish2(note, "", "", pubkeys, size);
        return ssb_note;
    };

    if let Some(rs) = ssbify(&note.annotations, &note.title, &note.url) {
        publish2(note, &rs.hash, "", pubkeys, size)
    } else {
        publish2(note, "", "", pubkeys, size)
    }
}

pub fn publish2(note: Note, hash: &str, markdown: &str, pubkeys: &str, size: usize) -> SsbNote {
    let note = Note {
        comments: hash.to_string(),
        annotations: markdown.chars().take(size).collect(),
        ..note
    };
    let note_json = json!(note).to_string();

    // eprintln!("node_json: {}", note_json);

    let mut child = Command::new("localnative-ssb-publish")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(pubkeys)
        .spawn()
        .expect("failed to execute process");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(note_json.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    eprintln!("status: {}", output.status);
    eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    let stderr = String::from_utf8_lossy(&output.stderr);
    // eprintln!("stderr: {}", stderr);

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        serde_json::from_str::<SsbNote>(&text).unwrap()
    } else if stderr.contains("Error: encoded message must not be larger than") {
        eprintln!("stderr: {}", stderr);
        publish2(note, hash, markdown, pubkeys, size - 100)
    } else {
        panic!("stderr: {}", stderr);
    }
}
