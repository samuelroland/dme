use std::fs::read_to_string;
mod highlight;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{
    format_html, markdown_to_html, markdown_to_html_with_plugins, parse_document, Arena,
    ComrakPlugins, Options, RenderOptionsBuilder,
};
use highlight::TreeSitterHighlighter;
use inkjet::{formatter, Highlighter, Language};

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

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
fn load_markdown_as_html(path: &str) -> Result<String, String> {
    read_to_string(path)
        .as_ref()
        .map(|c| convert_md_to_html(c))
        .map_err(|_| format!("Failed to load file {}", path).to_string())
}

fn convert_md_to_html(raw: &str) -> String {
    let highlighter = TreeSitterHighlighter {};
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.tasklist = true;

    let plugins = ComrakPlugins {
        render: comrak::RenderPlugins {
            codefence_syntax_highlighter: Some(&highlighter as &dyn SyntaxHighlighterAdapter),
            heading_adapter: None,
        },
    };
    markdown_to_html_with_plugins(raw, &options, &plugins)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_file_to_show, get_app_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
