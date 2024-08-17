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
use localnative_core::run_sync as run;
use std::fs::OpenOptions;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::str;

const LOG_FILE: &str = "debug.log";
const MAX_LOG_SIZE: u64 = 1 * 1024 * 1024; // 1MB

fn main() -> io::Result<()> {
    // Read the message length (first 4 bytes).
    let mut text_length_bytes = [0u8; 4];
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_exact(&mut text_length_bytes)?;

    let text_length: u32 = u32::from_ne_bytes(text_length_bytes);
    let text_length: usize = text_length as usize;
    eprintln!("text_length {:?}", text_length);
    log_to_file(format!("text_length {:?}", text_length))?;

    // Read the text (JSON object) of the message.
    let mut text_buf = vec![0; text_length];
    handle.read_exact(&mut text_buf)?;
    let text = str::from_utf8(&text_buf).expect("not utf8 string");
    eprintln!("text_buf {:?}", text);
    log_to_file(format!("text_buf {:?}", text))?;

    let response = run(text);
    eprintln!("response {:?}", response);
    log_to_file(format!("response {:?}", response))?;

    match send_message(&response) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error: {:?}", err);
            log_to_file(format!("Error: {:?}", err))?;
        }
    };
    Ok(())
}

// Sends message to the browser extension.
fn send_message(message: &str) -> io::Result<()> {
    let buf = message.as_bytes();
    let size = buf.len() as u32;

    let bytes: [u8; 4] = size.to_ne_bytes();

    let mut handle = io::stdout();
    // Write message size.
    handle.write_all(&bytes)?;
    // Write the message itself.
    handle.write_all(buf)?;
    handle.flush()?;
    Ok(())
}

fn log_to_file(message: String) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)?;

    let file_size = file.metadata()?.len();
    if file_size > MAX_LOG_SIZE {
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
    }

    writeln!(file, "{}", message)?;
    file.flush()?;
    Ok(())
}
