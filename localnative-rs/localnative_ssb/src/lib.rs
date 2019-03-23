/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
extern crate localnative_core;
use localnative_core::dirs;
use localnative_core::rusqlite;
#[macro_use]
extern crate serde_json;
use localnative_core::exe::get_sqlite_connection;
use localnative_core::{Note, SsbNote, Ssbify};
use std::io::Write;
use std::process::{Command, Stdio};
pub mod sync;

fn get_ssb_cli_path() -> String {
    format!(
        "{}/LocalNative/bin/localnative-nodejs-{}",
        dirs::home_dir().unwrap().to_str().unwrap(),
        "0.3.9"
    )
}

pub fn run_sync() {
    let id = whoami();
    let conn = get_sqlite_connection();
    sync::init_active_author(&conn, &id);
    sync::sync_to_ssb(&conn);
    sync::sync_all_to_db();
}

pub fn ssbify(content: &str, title: &str, url: &str) -> Option<Ssbify> {
    let mut child = Command::new(get_ssb_cli_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("ssbify")
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
    let output = Command::new(get_ssb_cli_path())
        .arg("ssb-whoami")
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
    let output = Command::new(get_ssb_cli_path())
        .arg("ssb-tail")
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

    let mut child = Command::new(get_ssb_cli_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("ssb-publish")
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
