#[tauri::command]
pub async fn input(input: String) -> String {
    let (tx, mut rx) = tauri::async_runtime::channel(1);

    tauri::async_runtime::spawn_blocking(move || {
        let _ = tx.blocking_send(localnative_core::run_sync(&input));
    });

    rx.recv()
        .await
        .unwrap_or_else(|| String::from(r#"{"error":"no reply from the core","code":"internal"}"#))
}

/// (Re)install the browser native-messaging manifests. Shares its
/// implementation with the Iced app via `localnative_hostinstall`.
#[tauri::command]
pub async fn fix_browser() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| localnative_hostinstall::WebKind::install_all(None))
        .await
        .map_err(|e| e.to_string())
}
