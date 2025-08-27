// Previewable implementation via a Comrak based Markdown parser
use super::preview::{Html, Previewable};
use super::tree_sitter_grammars::TreeSitterGrammarsManager;
use super::tree_sitter_highlight::TreeSitterHighlighter;
use comrak::html::escape;
use comrak::{adapters::SyntaxHighlighterAdapter, html};
use comrak::{markdown_to_html_with_plugins, ComrakPlugins, Options};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::RwLock;

// Global TreeSitterHighlighter cache indexed by language
static TSH_CACHE: Lazy<RwLock<HashMap<String, TreeSitterHighlighter>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static TREE_SITTER_GRAMMARS_FOLDER_VIA_ENV: Lazy<Option<String>> =
    Lazy::new(|| std::env::var("TREE_SITTER_GRAMMARS_FOLDER").ok());

pub struct ComrakParser {
    manager: TreeSitterGrammarsManager,
}

impl ComrakParser {
    /// Create a new ComrakParser with a default grammars folder or use the
    /// TREE_SITTER_GRAMMARS_FOLDER environment variable if defined
    pub fn new() -> Result<Self, String> {
        let manager = match &*TREE_SITTER_GRAMMARS_FOLDER_VIA_ENV {
            Some(folder) => {
                TreeSitterGrammarsManager::new_with_grammars_folder(PathBuf::from(folder.clone()))
            }
            None => TreeSitterGrammarsManager::new(),
        }?;

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
        let plugins = ComrakPlugins {
            render: comrak::RenderPlugins {
                codefence_syntax_highlighter: Some(self as &dyn SyntaxHighlighterAdapter),
                heading_adapter: None,
            },
        };
        Html::from(markdown_to_html_with_plugins(source, &options, &plugins))
    }
}

/// Implement a TreeSitterHighlighter integration on Comrak
// This is based on Syntect integration
// https://docs.rs/comrak/latest/src/comrak/plugins/syntect.rs.html#71-133
impl SyntaxHighlighterAdapter for ComrakParser {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        maybe_lang: Option<&str>,
        code: &str,
    ) -> io::Result<()> {
        let html = highlight_from_cached_highlighter(&self.manager, maybe_lang, code);
        let _ = output.write_all(html.to_safe_html_string().as_bytes());
        Ok(())
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
