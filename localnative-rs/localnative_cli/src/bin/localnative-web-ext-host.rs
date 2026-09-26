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

//! Browser-extension native-messaging host: read one JSON command from
//! stdin, run it, write one JSON reply to stdout.
//!
//! Nothing is logged — stderr carries only size/shape errors without any
//! note content, and there is no file logging at all. The commands the
//! extension may send are allow-listed; anything else is refused locally.

use localnative_core::run_sync as run;
use std::io::{self, Read, Write};
use std::str;

const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// The actions the popup uses. Sync, import and export stay desktop-side.
const ALLOWED_ACTIONS: &[&str] = &["insert", "insert-image", "search", "select", "delete"];

fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .init();

    // Read the message length (first 4 bytes, native endianness).
    let mut length_bytes = [0u8; 4];
    io::stdin().lock().read_exact(&mut length_bytes)?;
    let text_length = u32::from_ne_bytes(length_bytes) as usize;
    if text_length > MAX_MESSAGE_SIZE {
        // Content stays in stdin; reply and exit.
        eprintln!("message too large: {text_length} bytes");
        return send_message(
            r#"{"error":"Message exceeds maximum allowed size","code":"too-large"}"#,
        );
    }

    // Read the JSON command.
    let mut buffer = vec![0; text_length];
    io::stdin().lock().read_exact(&mut buffer)?;
    let text = match str::from_utf8(&buffer) {
        Ok(text) => text,
        Err(_) => {
            eprintln!("message was not valid UTF-8");
            return send_message(r#"{"error":"Invalid UTF-8 in message","code":"invalid-input"}"#);
        }
    };

    // Only known actions reach the core.
    let action = serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|value| {
            value
                .get("action")
                .and_then(|a| a.as_str())
                .map(str::to_owned)
        });
    match action.as_deref() {
        Some(action) if ALLOWED_ACTIONS.contains(&action) => {
            let response = run(text);
            // Never echo a failure's source on success paths; the core
            // envelope carries only an error message and code.
            send_message(&response)
        }
        other => {
            eprintln!("action not allowed: {:?}", other);
            send_message(
                r#"{"error":"This command is not available to the browser extension","code":"not-allowed"}"#,
            )
        }
    }
}

// Send one native-messaging frame to the extension.
fn send_message(message: &str) -> io::Result<()> {
    let bytes = message.as_bytes();
    let size = u32::try_from(bytes.len()).expect("response message exceeds u32::MAX bytes");
    let mut stdout = io::stdout();
    stdout.write_all(&size.to_ne_bytes())?;
    stdout.write_all(bytes)?;
    stdout.flush()
}
