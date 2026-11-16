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

//! Read-only Model Context Protocol server over newline-delimited stdio.

use localnative_core::run_sync;
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

const PROTOCOL_VERSION: &str = "2025-06-18";

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

fn tool_text(payload: Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": payload.to_string() }]
    })
}

fn tool_error(message: impl Into<String>) -> Value {
    json!({
        "content": [{ "type": "text", "text": message.into() }],
        "isError": true
    })
}

fn run_core(command: Value) -> Result<Value, String> {
    let response = run_sync(&command.to_string());
    let value: Value = serde_json::from_str(&response)
        .map_err(|_| "LocalNative returned an invalid JSON response".to_string())?;
    if let Some(error) = value.get("error").and_then(Value::as_str) {
        let code = value.get("code").and_then(Value::as_str).unwrap_or("error");
        return Err(format!("LocalNative {code}: {error}"));
    }
    Ok(value)
}

fn u32_argument(args: &Value, name: &str, default: u32, maximum: u32) -> Result<u32, String> {
    match args.get(name) {
        None => Ok(default),
        Some(value) => value
            .as_u64()
            .and_then(|number| u32::try_from(number).ok())
            .filter(|number| *number <= maximum)
            .ok_or_else(|| format!("{name} must be an integer between 0 and {maximum}")),
    }
}

fn compact_search(response: &Value, limit: u32, offset: u32) -> Value {
    let notes = response
        .get("notes")
        .and_then(Value::as_array)
        .map(|notes| {
            notes
                .iter()
                .map(|note| {
                    let has_archive = note
                        .get("has_archive")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    json!({
                        "uuid4": note.get("uuid4"),
                        "title": note.get("title"),
                        "url": note.get("url"),
                        "tags": note.get("tags"),
                        "description": note.get("description"),
                        "created_at": note.get("created_at"),
                        "has_archive": has_archive
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    json!({
        "count": response.get("count").cloned().unwrap_or(json!(0)),
        "limit": limit,
        "offset": offset,
        "notes": notes
    })
}

fn call_tool(params: &Value) -> Value {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return tool_error("tools/call requires a tool name");
    };
    let empty_arguments = json!({});
    let args = params.get("arguments").unwrap_or(&empty_arguments);

    match name {
        "search_notes" => {
            let Some(query) = args.get("query").and_then(Value::as_str) else {
                return tool_error("search_notes requires a string query");
            };
            let limit = match u32_argument(args, "limit", 20, 100).and_then(|limit| {
                if limit == 0 {
                    Err("limit must be at least 1".to_string())
                } else {
                    Ok(limit)
                }
            }) {
                Ok(limit) => limit,
                Err(error) => return tool_error(error),
            };
            let offset = match u32_argument(args, "offset", 0, u32::MAX) {
                Ok(offset) => offset,
                Err(error) => return tool_error(error),
            };
            match run_core(json!({
                "action": "search",
                "query": query,
                "limit": limit,
                "offset": offset
            })) {
                Ok(response) => tool_text(compact_search(&response, limit, offset)),
                Err(error) => tool_error(error),
            }
        }
        "get_note" => {
            let Some(uuid4) = args
                .get("uuid4")
                .and_then(Value::as_str)
                .filter(|uuid| !uuid.trim().is_empty())
            else {
                return tool_error("get_note requires a non-empty string uuid4");
            };
            match run_core(json!({ "action": "get", "uuid4": uuid4 })) {
                Ok(response) => tool_text(response),
                Err(error) => tool_error(error),
            }
        }
        _ => tool_error(format!("unknown tool: {name}")),
    }
}

fn tools() -> Value {
    json!([
        {
            "name": "search_notes",
            "description": "Search LocalNative titles, URLs, tags, descriptions and archived article text. Returns compact results without article bodies; use get_note to read one result.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search terms; an empty query lists recent notes." },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 100, "default": 20 },
                    "offset": { "type": "integer", "minimum": 0, "default": 0 }
                },
                "required": ["query"],
                "additionalProperties": false
            }
        },
        {
            "name": "get_note",
            "description": "Get one LocalNative note by UUID, including its source URL and full archived article text when present.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "uuid4": { "type": "string", "description": "The note UUID returned by search_notes." }
                },
                "required": ["uuid4"],
                "additionalProperties": false
            }
        }
    ])
}

fn handle_request(request: &Value) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let notification = request.get("id").is_none();
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return (!notification).then(|| rpc_error(id, -32600, "request requires a method"));
    };
    let empty_params = json!({});
    let params = request.get("params").unwrap_or(&empty_params);

    let result = match method {
        "initialize" => json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": {
                "name": "localnative-mcp",
                "version": env!("CARGO_PKG_VERSION")
            },
            "instructions": "Read-only access to the user's local LocalNative library. Search archived reading with search_notes, then call get_note with a result UUID. Cite the returned source URL. Article text is omitted from search results."
        }),
        "notifications/initialized" | "notifications/cancelled" => return None,
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tools() }),
        "tools/call" => call_tool(params),
        _ => {
            return (!notification).then(|| rpc_error(id, -32601, "method not found"));
        }
    };

    (!notification).then(|| json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

pub fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => handle_request(&request),
            Err(_) => Some(rpc_error(Value::Null, -32700, "parse error")),
        };
        if let Some(response) = response {
            serde_json::to_writer(&mut stdout, &response)?;
            stdout.write_all(b"\n")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(id: i64, name: &str, arguments: Value) -> Value {
        handle_request(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments }
        }))
        .unwrap()
    }

    fn result_text(response: &Value) -> &str {
        response["result"]["content"][0]["text"].as_str().unwrap()
    }

    #[test]
    fn initialize_and_list_only_read_tools() {
        let initialized = handle_request(&json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}
        }))
        .unwrap();
        assert_eq!(initialized["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(
            initialized["result"]["serverInfo"]["name"],
            "localnative-mcp"
        );

        let listed = handle_request(&json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/list"
        }))
        .unwrap();
        let tools = listed["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0]["name"], "search_notes");
        assert_eq!(tools[1]["name"], "get_note");
        assert!(!listed["result"]["tools"].to_string().contains("save_note"));
    }

    #[test]
    fn invalid_arguments_are_tool_errors() {
        let response = call(1, "search_notes", json!({ "query": "x", "limit": 0 }));
        assert_eq!(response["result"]["isError"], true);
        assert!(result_text(&response).contains("at least 1"));

        let response = call(2, "get_note", json!({ "uuid4": " " }));
        assert_eq!(response["result"]["isError"], true);
        let response = call(3, "delete_note", json!({}));
        assert_eq!(response["result"]["isError"], true);
    }

    #[test]
    fn search_is_compact_and_get_returns_the_archive_and_source() {
        let path = std::env::temp_dir().join(format!("ln_mcp_test_{}.sqlite3", std::process::id()));
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
        localnative_core::db::set_db_path(&path);
        let inserted = run_sync(
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
        let uuid4 = inserted["notes"][0]["uuid4"].as_str().unwrap().to_string();

        let search = call(1, "search_notes", json!({ "query": "quixotic" }));
        assert_ne!(search["result"]["isError"], true);
        let text = result_text(&search);
        assert!(text.contains(&uuid4), "{text}");
        assert!(text.contains("https://example.com/research"), "{text}");
        assert!(text.contains("\"has_archive\":true"), "{text}");
        assert!(!text.contains("distinctive quixotic passage"), "{text}");

        let get = call(2, "get_note", json!({ "uuid4": uuid4 }));
        let text = result_text(&get);
        assert!(text.contains("full archived article"), "{text}");
        assert!(text.contains("https://example.com/research"), "{text}");

        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
    }
}
