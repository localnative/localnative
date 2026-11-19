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

/// (Re)install the browser native-messaging manifests through the same shared
/// installer used by `localnative host install`.
#[tauri::command]
pub async fn fix_browser(allowed_origins: Option<Vec<String>>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        localnative_hostinstall::WebKind::try_install_all(allowed_origins)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}
