use std::path::PathBuf;

use dme_core::markdown_file_to_highlighted_html;

#[tauri::command]
/// Open given Markdown file or the default one provided as argument
/// or none otherwise
pub async fn open_markdown_file(mut path: String) -> Result<Option<String>, String> {
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
        Ok(Some(
            markdown_file_to_highlighted_html(&path)?.to_safe_html_string(),
        ))
    } else {
        Err(format!("File {path} doesn't exist !").to_string())
    }
}
