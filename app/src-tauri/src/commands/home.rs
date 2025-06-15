use serde::Serialize;

#[derive(Serialize)]
pub struct AppInfo {
    version: String,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
    }
}
