/*
    Local Native
    Copyright (C) 2018-2026  Yi Wang

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

use serde_json::{Value, json};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "localnative_cli_{name}_{}_{}",
            std::process::id(),
            nonce
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn join(&self, path: &str) -> PathBuf {
        self.0.join(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_localnative"))
}

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "status: {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn stdout_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "invalid JSON: {error}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn search(db: &Path, query: &str) -> Value {
    let output = success(
        cli()
            .arg("search")
            .arg(query)
            .arg("--db")
            .arg(db)
            .output()
            .unwrap(),
    );
    stdout_json(&output)
}

#[test]
fn unified_search_get_and_mcp_return_archives() {
    let scratch = Scratch::new("archive");
    let db = scratch.join("library.sqlite3");
    localnative_core::db::set_db_path(&db);
    let inserted = localnative_core::run_sync(
        &json!({
            "action": "insert",
            "title": "Local-first research",
            "url": "https://example.com/research",
            "tags": "local-first",
            "description": "A saved research article",
            "comments": "",
            "annotations": "full archived article with a distinctive quixotic passage",
            "limit": 10,
            "offset": 0,
            "is_public": false
        })
        .to_string(),
    );
    let inserted: Value = serde_json::from_str(&inserted).unwrap();
    let uuid = inserted["notes"][0]["uuid4"].as_str().unwrap();

    let output = search(&db, "quixotic");
    assert_eq!(output["count"], 1);
    assert_eq!(output["notes"][0]["uuid4"], uuid);
    assert_eq!(output["notes"][0]["annotations"], "");
    assert_eq!(output["notes"][0]["has_archive"], true);
    assert!(!output.to_string().contains("quixotic passage"));

    let output = success(
        cli()
            .arg("get")
            .arg(uuid)
            .arg("--db")
            .arg(&db)
            .output()
            .unwrap(),
    );
    let note = stdout_json(&output);
    assert_eq!(note["url"], "https://example.com/research");
    assert_eq!(
        note["annotations"],
        "full archived article with a distinctive quixotic passage"
    );

    let requests = [
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/call",
            "params": { "name": "search_notes", "arguments": { "query": "quixotic" } }
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": { "name": "get_note", "arguments": { "uuid4": uuid } }
        }),
    ];
    let mut child = cli()
        .arg("mcp")
        .arg("--db")
        .arg(&db)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for request in requests {
        writeln!(stdin, "{request}").unwrap();
    }
    drop(stdin);
    let output = success(child.wait_with_output().unwrap());
    let responses = output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 3);
    assert_eq!(
        responses[0]["result"]["serverInfo"]["name"],
        "localnative-mcp"
    );
    let search_text = responses[1]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert!(search_text.contains(uuid));
    assert!(!search_text.contains("quixotic passage"));
    let get_text = responses[2]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert!(get_text.contains("quixotic passage"));
    assert!(get_text.contains("https://example.com/research"));
}

#[test]
fn legacy_aliases_and_database_commands_keep_working() {
    let aliases = [
        env!("CARGO_BIN_EXE_localnative-mcp"),
        env!("CARGO_BIN_EXE_localnative-rpc-server"),
        env!("CARGO_BIN_EXE_localnative-rpc-client-sync"),
        env!("CARGO_BIN_EXE_localnative-rpc-client-stop-server"),
        env!("CARGO_BIN_EXE_localnative-import"),
        env!("CARGO_BIN_EXE_localnative-export"),
        env!("CARGO_BIN_EXE_localnative-export-db"),
        env!("CARGO_BIN_EXE_localnative-import-db"),
        env!("CARGO_BIN_EXE_localnative-upgrade"),
    ];
    for alias in aliases {
        let output = Command::new(alias).arg("--help").output().unwrap();
        assert!(
            output.status.success(),
            "{alias}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let scratch = Scratch::new("import_export");
    let db = scratch.join("library.sqlite3");
    let pocket = scratch.join("pocket.html");
    std::fs::write(
        &pocket,
        r#"<ul><li><a href="https://example.com/imported" time_added="1700000000" tags="rust">Imported Article</a></li></ul>"#,
    )
    .unwrap();
    success(
        Command::new(env!("CARGO_BIN_EXE_localnative-import"))
            .arg("--format")
            .arg("pocket")
            .arg(&pocket)
            .arg("--db")
            .arg(&db)
            .output()
            .unwrap(),
    );
    assert_eq!(search(&db, "Imported")["count"], 1);

    let markdown = scratch.join("markdown");
    success(
        cli()
            .arg("export")
            .arg("--output")
            .arg(&markdown)
            .arg("--db")
            .arg(&db)
            .output()
            .unwrap(),
    );
    assert_eq!(std::fs::read_dir(&markdown).unwrap().count(), 1);

    let backup = scratch.join("backup.sqlite3");
    success(
        Command::new(env!("CARGO_BIN_EXE_localnative-export-db"))
            .arg("--output")
            .arg(&backup)
            .arg("--db")
            .arg(&db)
            .output()
            .unwrap(),
    );
    assert!(backup.is_file());

    let merged = scratch.join("merged.sqlite3");
    success(
        cli()
            .arg("import-db")
            .arg("--input")
            .arg(&backup)
            .arg("--db")
            .arg(&merged)
            .output()
            .unwrap(),
    );
    assert_eq!(search(&merged, "Imported")["count"], 1);

    let output = success(
        Command::new(env!("CARGO_BIN_EXE_localnative-upgrade"))
            .arg("--db")
            .arg(&merged)
            .output()
            .unwrap(),
    );
    assert_eq!(stdout_json(&output), "Upgrade completed");
}

#[cfg(not(target_os = "windows"))]
#[test]
fn host_install_preserves_native_messaging() {
    let scratch = Scratch::new("host");
    let home = scratch.join("home");
    let db = scratch.join("browser.sqlite3");
    std::fs::create_dir_all(&home).unwrap();
    success(
        cli()
            .arg("host")
            .arg("install")
            .env("HOME", &home)
            .output()
            .unwrap(),
    );

    #[cfg(target_os = "macos")]
    let manifest = home
        .join("Library/Application Support/Google/Chrome/NativeMessagingHosts")
        .join("app.localnative.json");
    #[cfg(target_os = "linux")]
    let manifest = home
        .join(".config/google-chrome/NativeMessagingHosts")
        .join("app.localnative.json");
    let manifest: Value = serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
    assert_eq!(manifest["name"], "app.localnative");
    assert_eq!(manifest["type"], "stdio");
    let host = PathBuf::from(manifest["path"].as_str().unwrap());
    assert!(
        host.is_file(),
        "installed host is missing: {}",
        host.display()
    );

    let message = json!({
        "action": "insert",
        "title": "Browser archive",
        "url": "https://example.com/browser",
        "tags": "browser",
        "description": "Saved through native messaging",
        "comments": "",
        "annotations": "browser article body with a memorable zephyr sentence",
        "limit": 10,
        "offset": 0,
        "is_public": false
    })
    .to_string();
    let mut child = Command::new(host)
        .env("HOME", &home)
        .env("LOCALNATIVE_DB", &db)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(&(message.len() as u32).to_ne_bytes())
        .unwrap();
    stdin.write_all(message.as_bytes()).unwrap();
    drop(stdin);
    let output = success(child.wait_with_output().unwrap());
    let length = u32::from_ne_bytes(output.stdout[..4].try_into().unwrap()) as usize;
    assert_eq!(length, output.stdout.len() - 4);
    let inserted: Value = serde_json::from_slice(&output.stdout[4..]).unwrap();
    assert_eq!(inserted["count"], 1);
    assert_eq!(inserted["notes"][0]["has_archive"], true);

    let result = search(&db, "zephyr");
    assert_eq!(result["count"], 1);
    assert_eq!(result["notes"][0]["url"], "https://example.com/browser");
    assert_eq!(result["notes"][0]["annotations"], "");
}
