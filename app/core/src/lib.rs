use std::fs::read_to_string;
mod highlight;
mod util;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{
    format_html, markdown_to_html, markdown_to_html_with_plugins, parse_document, Arena,
    ComrakPlugins, Options, RenderOptionsBuilder,
};
use highlight::TreeSitterHighlighter;
use inkjet::{formatter, Highlighter, Language};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub fn load_markdown_as_html(path: &str) -> Result<String, String> {
    read_to_string(path)
        .as_ref()
        .map(|c| convert_md_to_html(c))
        .map_err(|_| format!("Failed to load file {}", path).to_string())
}

pub fn convert_md_to_html(raw: &str) -> String {
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
