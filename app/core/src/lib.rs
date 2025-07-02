use std::fs::read_to_string;
pub mod export;
pub mod preview;
pub mod search;
pub mod theming;
pub mod util;

use preview::{
    comrak::ComrakParser,
    preview::{Html, Previewable},
};
use theming::{
    helix::ALL_HIGHLIGHT_NAMES_SUPPORTED_BY_HELIX,
    renderer::Renderer,
    theme::{Theme, DEFAULT_THEME},
};

/// Given a Markdown file convert it to a full Html document that can be used as a .html file
/// directly, with all the Tre-Sitter highlighting for code blocks and a default code theme applied
pub fn markdown_to_highlighted_html(path: &str) -> Result<Html, String> {
    let content = read_to_string(path)
        .map_err(|e| "Couldn't find given file: ".to_string() + &e.to_string())?;
    let theme = Theme::from_helix(DEFAULT_THEME, ALL_HIGHLIGHT_NAMES_SUPPORTED_BY_HELIX)
        .map_err(|e| e.to_string())?;
    let renderer = Renderer::new(&theme);
    let css = renderer.css();
    let parser = ComrakParser::new()?;
    let html = parser.to_html(&content);
    let html = format!("<style>{}</style>\n\n{}", css, html.as_string());
    Ok(Html(html))
}
