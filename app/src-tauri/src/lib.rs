use std::fs::read_to_string;

use comrak::{markdown_to_html, Options};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn load_markdown_as_html(path: &str) -> Result<String, String> {
    // Some("salut !".to_string())
    read_to_string(path)
        .as_ref()
        .map(|c| convert_md_to_html(c))
        .map_err(|_| "Failed to load file".to_string())
}

fn convert_md_to_html(raw: &str) -> String {
    let mut options = Options::default();
    options.extension.table = true;
    markdown_to_html(raw, &options)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_markdown_as_html])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
