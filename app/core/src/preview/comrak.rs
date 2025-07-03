// Previewable implementation via a Comrak based Markdown parser
use super::preview::{Html, Previewable};
use super::tree_sitter_grammars::TreeSitterGrammarsManager;
use super::tree_sitter_highlight::TreeSitterHighlighter;
use ammonia::Builder;
use comrak::html::escape;
use comrak::{adapters::SyntaxHighlighterAdapter, html};
use comrak::{markdown_to_html_with_plugins, ComrakPlugins, Options};
use maplit::{hashmap, hashset};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use tree_sitter_loader::Loader;

pub struct ComrakParser {
    manager: TreeSitterGrammarsManager,
}

impl ComrakParser {
    /// Create a new ComrakParser with a default grammars folder or use the
    /// TREE_SITTER_GRAMMARS_FOLDER environment variable if defined
    pub fn new() -> Result<Self, String> {
        let manager = TreeSitterGrammarsManager::new()?;
        Ok(ComrakParser { manager })
    }

    /// A ComrakParser parser but with a different grammars folder than default
    /// or the version defined in env, as this is not a good solution for testing
    /// Public only for this crate as only useful for testing
    pub(crate) fn new_with_configurable_grammars_folder(folder: String) -> Result<Self, String> {
        let manager =
            TreeSitterGrammarsManager::new_with_grammars_folder(PathBuf::from(folder.clone()))?;
        Ok(ComrakParser { manager })
    }
}

impl Previewable for ComrakParser {
    fn to_html(&self, source: &str) -> Html {
        let mut options = Options::default();
        options.extension.table = true; // Enable tables
        options.extension.tasklist = true; // Enable list of tasks
        options.extension.autolink = true; // Enable creating links automatically for URLs in text
        options.render.unsafe_ = true; // we take care of it with ammonia sanitizer just below
        let plugins = ComrakPlugins {
            render: comrak::RenderPlugins {
                codefence_syntax_highlighter: Some(self as &dyn SyntaxHighlighterAdapter),
                heading_adapter: None,
            },
        };

        let unsafe_html = &markdown_to_html_with_plugins(source, &options, &plugins);

        // Make the HTML safe by calling
        let mut cleaner = Builder::default();
        let authorized_tags_attribute = hashmap! {
            "code" => hashset!{"class"}, // authorize the class attribute for <code> because we need
            "span" => hashset!{"class"} // same as for <code>
        };
        cleaner.tag_attributes(authorized_tags_attribute);
        Html::from(cleaner.clean(unsafe_html).to_string())
    }
}

/// Implement a TreeSitterHighlighter integration on Comrak
// This is based on Syntect integration
// https://docs.rs/comrak/latest/src/comrak/plugins/syntect.rs.html#71-133
impl SyntaxHighlighterAdapter for ComrakParser {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> io::Result<()> {
        // Do not highlight in case there is no lang or it's empty
        if lang.is_none_or(|v| v.trim().is_empty()) {
            let mut escaped = Vec::new();
            escape(&mut escaped, code.as_bytes()).unwrap_or_default();
            output.write_all(escaped.as_slice())
        } else {
            let mut loader = Loader::new().map_err(std::io::Error::other)?;
            let highlighter =
                TreeSitterHighlighter::new(&mut loader, lang.unwrap_or_default(), &self.manager);
            // If lang might be supported or not
            match highlighter {
                Ok(highlighter) => {
                    output.write_all(highlighter.highlight(code).as_string().as_bytes())
                }
                Err(_) => {
                    let mut escaped = Vec::new();
                    match escape(&mut escaped, code.as_bytes()) {
                        Ok(_) => output.write_all(escaped.as_slice()),
                        Err(_) => output.write_all("failed to escape code sorry...".as_bytes()),
                    }
                }
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
