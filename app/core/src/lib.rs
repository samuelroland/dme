use std::fs::read_to_string;
mod preview;
mod util;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{
    format_html, markdown_to_html, markdown_to_html_with_plugins, parse_document, Arena,
    ComrakPlugins, Options, RenderOptionsBuilder,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub fn load_markdown_as_html(path: &str) -> Result<String, String> {
    Ok("TODO".to_string())
}
