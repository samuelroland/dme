use std::fs::read_to_string;
pub mod export;
pub mod preview;
pub mod search;
pub mod theming;
mod util;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{
    format_html, markdown_to_html, markdown_to_html_with_plugins, parse_document, Arena,
    ComrakPlugins, Options, RenderOptionsBuilder,
};
