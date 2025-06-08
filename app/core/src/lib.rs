use std::fs::read_to_string;

use preview::{
    comrak::ComrakParser,
    preview::{Html, Previewable},
    tree_sitter_highlight::TreeSitterHighlighter,
};

pub mod preview;
pub mod util;

/// Given a Markdown file convert it to a full Html document that can be used as a .html file
/// directly, with all the Tre-Sitter highlighting for code blocks and a default code theme applied
pub fn markdown_to_highlighted_html(path: &str) -> Result<Html, String> {
    let content = read_to_string(path)
        .map_err(|e| "Couldn't find given file: ".to_string() + &e.to_string())?;
    let parser = ComrakParser::new()?;
    let html = parser.to_html(&content);
    Ok(html)
}
