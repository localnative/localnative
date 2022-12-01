#[tauri::command]
pub fn input(input: String) -> String {
    localnative_core::exe::run(&input)
}
