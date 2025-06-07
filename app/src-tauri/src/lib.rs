use dme_core::load_markdown_as_html;
use std::fs::read_to_string;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Serialize)]
struct AppInfo {
    version: String,
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
    }
}

#[tauri::command]
fn get_file_to_show() -> Option<Result<String, String>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        None
    } else {
        let file = &args[1];
        Some(load_markdown_as_html(file))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_file_to_show, get_app_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
