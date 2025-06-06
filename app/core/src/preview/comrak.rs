// Previewable implementation via a Comrak based Markdown parser
use super::preview::{Html, Previewable};
use super::tree_sitter::TreeSitterHighlighter;
use comrak::{adapters::SyntaxHighlighterAdapter, html};
use comrak::{markdown_to_html, markdown_to_html_with_plugins, ComrakPlugins, Options};
use std::collections::HashMap;
use std::io::{self, Write};

pub struct ComrakParser<'a> {
    source: &'a str,
}

impl<'a> ComrakParser<'a> {
    pub fn new(content: &'a str) -> Self {
        ComrakParser { source: content }
    }
}

impl<'a> Previewable<'a> for ComrakParser<'_> {
    fn to_html(&self) -> Html {
        let mut options = Options::default();
        options.extension.table = true; // Enable tables
        options.extension.tasklist = true; // Enable list of tasks
        options.extension.autolink = true; // Enable creating links automatically for URLs in text
        let highlighter = TreeSitterAdapter {};
        let plugins = ComrakPlugins {
            render: comrak::RenderPlugins {
                codefence_syntax_highlighter: Some(&highlighter as &dyn SyntaxHighlighterAdapter),
                heading_adapter: None,
            },
        };
        Html::from(markdown_to_html_with_plugins(
            self.source,
            &options,
            &plugins,
        ))
    }
}

/// Implement a TreeSitterAdapter to be able to use TreeSitterHighlighter
/// in code blocks extracted by Comrak
struct TreeSitterAdapter {}
// This is based on Syntect integration
// https://docs.rs/comrak/latest/src/comrak/plugins/syntect.rs.html#71-133
impl SyntaxHighlighterAdapter for TreeSitterAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> io::Result<()> {
        // Do not highlight in case there is no lang or it's empty
        if lang.is_none_or(|v| v.trim().is_empty()) {
            output.write_all(code.as_bytes())
        } else {
            let lang = TreeSitterHighlighter::normalize_lang(lang.unwrap_or_default());
            let highlighter = TreeSitterHighlighter::new(lang);
            // If lang might be supported or not
            match highlighter {
                Ok(highlighter) => {
                    output.write_all(highlighter.highlight(code).as_string().as_bytes())
                }
                Err(_) => output.write_all(code.as_bytes()),
            }
        }
    }

    // Just use <pre> and <code> tags as usual, without anything special
    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        let _ = html::write_opening_tag(output, "pre", attributes);
        Ok(())
    }
    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        html::write_opening_tag(output, "code", attributes)
    }
}
