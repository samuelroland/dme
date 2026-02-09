use std::path::PathBuf;

use dme_core::markdown_file_to_highlighted_html;
use dme_core::preview::preview::ImageUrlRewriteMode;

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

    let pathbuf = PathBuf::from(&path);
    if path.is_empty() {
        Ok(None)
    } else if pathbuf.exists() {
        let pwd: PathBuf = PathBuf::from(".");
        let parent_path = PathBuf::from(path.clone())
            .parent()
            .unwrap_or_else(|| &pwd)
            .to_string_lossy()
            .to_string();
        Ok(Some(
            markdown_file_to_highlighted_html(&pathbuf)?
                .set_image_rewrite(ImageUrlRewriteMode::TauriFullPath(parent_path))
                .to_safe_html_string(),
        ))
    } else {
        Err(format!("File {path} doesn't exist !").to_string())
    }
}
