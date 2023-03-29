#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod command;
mod init;

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            command::input,
            command::local_ip,
            command::test_sync_server_addr,
            command::fix_browser
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
