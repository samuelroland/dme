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

/// The high level entrypoint to access a cached TreeSitterHighlighter and highlight a given piece of code
/// If the grammar is not installed or cannot be used, the original code is used in its HTML
/// escaped form to avoid being changed in sanitization
pub fn highlight_code_from_cached_highlighter(
    manager: &TreeSitterGrammarsManager,
    maybe_lang: Option<&str>,
    code: &str,
) -> Html {
    if let Some(lang) = maybe_lang {
        if !lang.is_empty() {
            let owned_lang = lang.to_owned();
            let cache = TSH_CACHE.read().unwrap();
            let highlighter = cache.get(&owned_lang);

            match highlighter {
                // We have a highlighter in cache, juse use it
                Some(h_cached) => {
                    return h_cached.highlight(code);
                }
                None => {
                    // Otherwise create a new one, use it and save it in CACHE
                    let new_h = TreeSitterHighlighter::new(&owned_lang, manager);
                    if let Ok(valid_new_h) = new_h {
                        let result = valid_new_h.highlight(code);
                        drop(cache);
                        let mut cache = TSH_CACHE.write().unwrap();
                        cache.insert(owned_lang, valid_new_h);
                        drop(cache);
                        return result;
                    }
                }
            }
        }
    }

    // If we reach this point, we need to take the original code without highlighting
    // BUT we need to escape it to support things like "#include <iostream>" and not have it
    // removed by the sanitization

    let mut escaped = Vec::new();
    if escape(&mut escaped, code.as_bytes()).is_err() {
        let _ = escaped.write_all("failed to escape code sorry...".as_bytes());
    }
    Html::from(String::from_utf8(escaped).unwrap_or("Invalid non UTF8 escaped code...".to_string()))
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
        let html = highlight_code_from_cached_highlighter(&self.manager, maybe_lang, code);
        // TODO: refactor this to avoid calling to_safe_html_string on each code snippet + on the whole final document
        // How can we call it only at the end ?
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
