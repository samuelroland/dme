// Tree-sitter highlighter via external grammars on disk
// This implementation has been inspired/helped by this article
// https://dotat.at/@/2025-03-30-hilite.html
// and the associated implementation
// https://dotat.at/cgi/git/wwwdotat.git/blob/HEAD:/src/hilite.rs

use std::{fs::read_to_string, path::PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;
use tree_sitter::Parser;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, Highlighter, HtmlRenderer};
use tree_sitter_loader::{CompileConfig, Loader};

use super::{preview::Html, tree_sitter_grammars::TreeSitterGrammarsManager};

// TODO: refactor with dynamic path from configuration
const HIGHLIGHT_QUERIES_PATH: &str = "queries/highlights.scm";

static HIGHLIGHT_NAMES_PARSER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\@[A-Za-z\.]+").unwrap());

/// A highlighter for a specific language, once loaded it can highlight multiple code snippets of
/// the same programming language
pub struct TreeSitterHighlighter<'a> {
    /// The language identifier
    lang: &'a str,
    repos_path: PathBuf,
    highlight_config: HighlightConfiguration,
    parser: Parser,
}

impl<'a> TreeSitterHighlighter<'a> {
    /// Try to create a new highlighter based on a
    pub fn new(lang: &'a str) -> Result<Self, String> {
        // Note: we making the supposition that the lang is in the folder name, for now
        let repos_path = TreeSitterGrammarsManager::get_repos_for_lang(lang)?
            .path()
            .clone();
        if repos_path.exists() {
            let mut loader = Loader::new().map_err(|e| e.to_string())?;
            // Even if the repos exists, it might not be a valid Tree-Sitter syntax
            let language = loader
                .load_language_at_path(CompileConfig::new(&repos_path, None, None))
                .map_err(|e| e.to_string())?;

            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&language).unwrap();

            // Note: tree-sitter.json contains an array of `grammars` which could be more than one
            // grammar sometimes (typescript -> typescript, tsx and flow. xml -> xml and dtd)
            // For now, we only support the first entry.
            let language_configs = loader
                .find_language_configurations_at_path(&repos_path, false)
                .map_err(|e| e.to_string())?;
            let first = language_configs
                .first()
                .ok_or("Given path has no grammar at all in tree-sitter.json configuration")?;

            let highlights_queries = first
                .highlights_filenames
                .clone()
                .ok_or("No highlightings files detected !!")?
                .iter()
                .map(|path| read_to_string(path).unwrap_or_default())
                .collect::<Vec<String>>()
                .join("\n");

            let highlight_config =
                HighlightConfiguration::new(language, lang, &highlights_queries, "", "")
                    .map_err(|e| e.to_string())?;

            Ok(TreeSitterHighlighter {
                lang,
                repos_path,
                highlight_config,
                parser,
            })
        } else {
            Err("The grammar {lang} is not installed locally".to_string())
        }
    }

    /// Just get the language of the highlighter defined via new()
    pub fn get_lang(&self) -> &'a str {
        self.lang
    }

    /// Special callback passed to HtmlRenderer::render that take a token with a highlight name
    /// attributed via the usize index inside the vector of self.highlight_config.names()
    /// This callback needs the context of self.highlight_config so it is wrapped in this function
    fn get_callback_to_apply_highlight_on_token(
        &'a self,
    ) -> impl Fn(tree_sitter_highlight::Highlight, &mut std::vec::Vec<u8>) + 'a {
        let callback = |highlight: Highlight, output: &mut Vec<u8>| {
            output.extend_from_slice(
                format!(
                    "class='{}'",
                    self.highlight_config
                        .names()
                        .get(highlight.0)
                        // highlight is just a usize value indexing our vector of highlight names
                        .unwrap()
                        .replace(".", " ")
                )
                .as_bytes(),
            );
        };
        callback
    }

    /// Given a code content + a Tree-sitter grammar directory for this language
    /// dynamically load this Tree-sitter parser and render HTML back
    /// It will detect all highlight names present in the queries files
    pub fn highlight(&self, code: &str) -> Html {
        let mut highlighter = Highlighter::new();

        // Do the final highlighting of given code
        let highlights = highlighter
            .highlight(&self.highlight_config, code.as_bytes(), None, |_| None)
            .unwrap();

        let mut renderer = HtmlRenderer::new();
        renderer
            .render(
                highlights,
                code.as_bytes(),
                &self.get_callback_to_apply_highlight_on_token(),
            )
            .unwrap();
        Html(
            String::from_utf8(renderer.html)
                .unwrap_or("Rendered HTML is not a valid UTF8, could not render.".to_string()),
        )
    }

    /// Normalise code block given lang to a set of known equivalence
    /// like js -> javascript, vuejs -> vue
    pub fn normalize_lang(given: &'a str) -> &'a str {
        match given {
            "bash" | "sh" | "shell" => "bash",
            "js" => "javascript",
            "rs" => "rust",
            "rb" => "ruby",
            "kt" => "kotlin",
            "vuejs" => "vue",
            "py" => "python",
            "md" => "markdown",
            "hs" => "haskell",
            "ts" => "typescript",
            other => other,
        }
    }
}
