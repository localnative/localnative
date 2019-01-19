use std::io;
use std::io::{Read, Write};
use std::mem::transmute;
use std::str;
extern crate localnative_core;
extern crate localnative_ssb;
use localnative_core::exe::run;
use localnative_core::serde_json;
use localnative_core::Cmd;
use localnative_ssb as ssb;

fn main() -> io::Result<()> {
    // Read the message length (first 4 bytes).
    let mut text_length_bytes = [0u8; 4];
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_exact(&mut text_length_bytes)?;

    let text_length: u32 = unsafe { transmute(text_length_bytes) };
    let text_length: usize = text_length as usize;
    eprintln!("text_length {:?}", text_length);

    // Read the text (JSON object) of the message.
    // let mut text_buf = vec![0; text_length as usize];
    let mut text_buf = vec![0; text_length];
    handle.read_exact(&mut text_buf)?;
    let text = str::from_utf8(&text_buf).expect("not utf8 string");
    eprintln!("text_buf {:?}", text);

    if let Ok(cmd) = serde_json::from_str::<Cmd>(text) {
        match cmd.action.as_ref() {
            "ssb-sync" => {
                ssb::run_sync();
                match send_message(r#"{"ssb-sync": "done"}"#) {
                    Ok(_) => (),
                    Err(err) => eprintln!("Error: {:?}", err),
                };
            }
            _ => {
                let response = run(text);
                eprintln!("response {:?}", response);
                match send_message(&response) {
                    Ok(_) => (),
                    Err(err) => eprintln!("Error: {:?}", err),
                };
            }
        }
    }
    Ok(())
}

// Sends message to the browser extension.
fn send_message(message: &str) -> io::Result<()> {
    let buf = message.as_bytes();
    let size = buf.len() as u32;

    let bytes: [u8; 4] = if cfg!(target_endian = "little") {
        eprintln!("LE");
        unsafe { transmute(size.to_le()) }
    } else {
        eprintln!("BE");
        unsafe { transmute(size.to_be()) }
    };

    let mut handle = io::stdout();
    // Write message size.
    handle.write(&bytes)?;
    // Write the message itself.
    handle.write(buf)?;
    handle.flush()?;
    Ok(())
}
