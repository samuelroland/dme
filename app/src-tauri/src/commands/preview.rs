use std::path::PathBuf;

use dme_core::{markdown_to_highlighted_html, preview::preview::Html};

#[tauri::command]
/// Open given Markdown file or the default one provided as argument
/// or none otherwise
pub fn open_markdown_file(mut path: String) -> Result<Option<Html>, String> {
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
