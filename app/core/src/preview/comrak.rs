use crate::preview::math::MathRenderer;

// Previewable implementation via a Comrak based Markdown parser
use super::preview::{Html, Previewable};
use super::tree_sitter_grammars::TreeSitterGrammarsManager;
use super::tree_sitter_highlight::TreeSitterHighlighter;
use comrak::html::escape;
use comrak::nodes::NodeValue;
use comrak::options::Plugins;
use comrak::{adapters::SyntaxHighlighterAdapter, html};
use comrak::{format_html_with_plugins, parse_document, Arena, Options};
use core::fmt;
use once_cell::sync::Lazy;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

// Global TreeSitterHighlighter cache indexed by language
static TSH_CACHE: Lazy<RwLock<HashMap<String, TreeSitterHighlighter>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static TREE_SITTER_GRAMMARS_FOLDER_VIA_ENV: Lazy<Option<String>> =
    Lazy::new(|| std::env::var("TREE_SITTER_GRAMMARS_FOLDER").ok());

// Global MathRenderer to avoid recreating a typst world all the time
// and using an internal prefix id counter globally unique
static MATH_RENDERER: Lazy<MathRenderer> = Lazy::new(MathRenderer::init);

/// A prefix for security purpose, to avoid being able to create arbitrary ID in the DOM from the Markdown headings.
/// This is a way to make it safe if some JavaScript code is relying on the id attribute of something outside of the article.
pub const HEADER_IDS_SECURITY_PREFIX: &str = "h-";

pub const FRONT_MATTER_DELIMITER: &str = "---";

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
        // Configuring Comrak options and plugins before running them
        let mut options = Options::default();
        options.extension.table = true; // Enable tables
        options.extension.tasklist = true; // Enable list of tasks
        options.extension.autolink = true; // Enable creating links automatically for URLs in text
        options.extension.math_dollars = true;
        options.extension.front_matter_delimiter = Some(FRONT_MATTER_DELIMITER.into());
        options.extension.header_ids = Some(HEADER_IDS_SECURITY_PREFIX.into());

        options.render.r#unsafe = true; // Unable unsafe mode to allow HTML to go through. To avoid XSS, we take care of it with ammonia sanitizer in the Html wrapper type
        let plugins = Plugins {
            render: comrak::options::RenderPlugins {
                codefence_syntax_highlighter: Some(self as &dyn SyntaxHighlighterAdapter),
                heading_adapter: None,
            },
        };

        // Structure based on code inside markdown_to_html_with_plugins()
        let arena = Arena::new();
        let root = parse_document(&arena, source, &options);

        for node in root.descendants() {
            let node_borrow = &mut node.data.borrow_mut();
            if let NodeValue::Math(node_math) = &node_borrow.value {
                let math_exp = &node_math.literal;
                let maybe_svg = MATH_RENDERER.convert_math_expression_into_svg(math_exp);
                let mut result = match maybe_svg {
                    Ok(svg) => svg,
                    Err(err) => format!("<span class='parse-error'>{err}</span>"),
                };
                // We want to wrap the block inside a paragraph as the SVG itself is not different for inline or block
                // Adding these class allow to easily style them differently
                // Note: The attribute display_math is true when this is a block math expressions. display_math: opendollars == 2,
                let (tag, css_class) = if node_math.display_math {
                    ("p", "math-block")
                } else {
                    ("span", "math-inline")
                };

                result = format!("<{tag} class='{css_class}'>{result}</{tag}>");
                node_borrow.value = NodeValue::HtmlInline(result);
            }
        }

        // Normal
        let mut rendered_html = String::default();
        format_html_with_plugins(root, &options, &mut rendered_html, &plugins).unwrap();
        Html::from(rendered_html)
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

    let mut escaped = String::default();
    if escape(&mut escaped, code).is_err() {
        escaped.push_str("failed to escape code sorry...");
    }
    Html::from(escaped)
}

/// Implement a TreeSitterHighlighter integration on Comrak
// This is based on Syntect integration
// https://docs.rs/comrak/latest/src/comrak/plugins/syntect.rs.html#71-133
impl SyntaxHighlighterAdapter for ComrakParser {
    fn write_highlighted(
        &self,
        output: &mut dyn std::fmt::Write,
        maybe_lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        let html = highlight_code_from_cached_highlighter(&self.manager, maybe_lang, code);
        // TODO: refactor this to avoid calling to_safe_html_string on each code snippet + on the whole final document
        // How can we call it only at the end ?
        let _ = output.write_str(&html.to_safe_html_string());
        Ok(())
    }

    // Just use <pre> and <code> tags as usual, without anything special
    fn write_pre_tag(
        &self,
        output: &mut dyn std::fmt::Write,
        attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        let _ = html::write_opening_tag(output, "pre", attributes);
        Ok(())
    }

    fn write_code_tag(
        &self,
        output: &mut dyn std::fmt::Write,
        attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        html::write_opening_tag(output, "code", attributes)
    }
}
