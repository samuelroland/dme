use std::{env::current_dir, path::PathBuf, sync::mpsc};

use dme_core::{
    markdown_to_highlighted_html,
    preview::preview::Html,
    search::{
        disk::DiskResearcher,
        search::{ResearchResult, Researcher},
    },
};
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
/// Open given Markdown file or the default one provided as argument
/// or none otherwise
fn open_markdown_file(mut path: String) -> Result<Option<Html>, String> {
    println!("{path:?}");
    if path.is_empty() {
        path = {
            let args: Vec<String> = std::env::args().collect();
            if args.len() < 2 {
                String::default()
            } else {
                args[1].clone()
            }
        }
    }

    if path.is_empty() {
        Ok(None)
    } else if PathBuf::from(&path).exists() {
        Ok(Some(markdown_to_highlighted_html(&path)?))
    } else {
        Err(format!("File {path} doesn't exist !").to_string())
    }
}

#[tauri::command]
fn run_search(search: String) -> Result<Vec<ResearchResult>, String> {
    let mut disk_search = DiskResearcher::new(
        current_dir()
            .map_err(|e| e.to_string())?
            .join("../..")
            .to_str()
            .unwrap_or_default()
            .to_string(),
    );
    disk_search.start();
    let (tx, rx) = mpsc::channel::<ResearchResult>();
    let results = disk_search.search(&search, 20, Some(tx.clone()));
    Ok(results)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            run_search,
            open_markdown_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
