// Tree-sitter highlighter via external grammars on disk
// This implementation has been inspired/helped by this article
// https://dotat.at/@/2025-03-30-hilite.html
// and the associated implementation
// https://dotat.at/cgi/git/wwwdotat.git/blob/HEAD:/src/hilite.rs

use pretty_assertions::{assert_eq, assert_ne};
use std::fs::read_to_string;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, Highlighter, HtmlRenderer};
use tree_sitter_loader::{CompileConfig, LanguageConfiguration, Loader};

use super::{preview::Html, tree_sitter_grammars::TreeSitterGrammarsManager};

/// A highlighter for a specific language, once loaded it can highlight multiple code snippets of
/// the same programming language
pub struct TreeSitterHighlighter<'a> {
    /// The language identifier
    lang: String,
    /// The highlighting configuration containing highlight queries,
    /// injections queries and local queries.
    highlight_config: &'a HighlightConfiguration,
}

impl<'a> TreeSitterHighlighter<'a> {
    /// Try to create a new highlighter based on an external loader
    /// A loader created with Loader::new() is fine
    /// The language to highlight, a grammar for this must be installed or it will fail
    /// The manager is used to get the grammar folder for this language
    pub fn new(
        loader: &'a mut Loader,
        lang: &String,
        manager: &TreeSitterGrammarsManager,
    ) -> Result<Self, String> {
        let lang = Self::normalize_lang(&lang).to_string();

        // Note: we making the supposition that the lang is in the folder name, for now
        let repos_path = manager.get_repos_for_lang(&lang)?.path().clone();
        if repos_path.exists() {
            // Even if the repos exists, it might not be a valid Tree-Sitter syntax
            let language = loader
                .load_language_at_path(CompileConfig::new(&repos_path.join("src"), None, None))
                .map_err(|e| e.to_string())?;

            // Note: tree-sitter.json contains an array of `grammars` which could be more than one
            // grammar sometimes (typescript -> typescript, tsx and flow. xml -> xml and dtd)
            // For now, we only support the first entry.
            let language_configs = loader
                .find_language_configurations_at_path(&repos_path, false)
                .map_err(|e| e.to_string())?;
            let first = language_configs
                .first()
                .ok_or("Given path has no grammar at all in tree-sitter.json configuration")?;

            // Generate a highlight configuration based on the language
            // and no additionnal paths to queries files
            let highlight_config = first
                .highlight_config(language, None)
                .map_err(|e| e.to_string())?
                .ok_or("No highlighting queries defined for the language")?;

            Ok(TreeSitterHighlighter {
                lang,
                highlight_config,
            })
        } else {
            Err("The grammar {lang} is not installed locally".to_string())
        }
    }

    /// Just get the language of the highlighter defined via new()
    pub fn get_lang(&self) -> String {
        self.lang.clone()
    }

    /// Special callback passed to HtmlRenderer::render that take a token with a highlight name
    /// attributed via the usize index inside the vector of self.highlight_config.names()
    /// This callback needs the context of self.highlight_config this is why it is wrapped in this function
    fn get_callback_to_apply_highlight_on_token(
        &'a self,
    ) -> impl Fn(tree_sitter_highlight::Highlight, &mut std::vec::Vec<u8>) + 'a {
        let callback = |highlight: Highlight, output: &mut Vec<u8>| {
            output.extend_from_slice(
                format!(
                    "class='{}'",
                    self.highlight_config
                        .names() // all highlight names used in queries files
                        .get(highlight.0)
                        // highlight is just a usize value indexing our vector of highlight names
                        .unwrap_or(&"")
                        .replace(".", " ") // change "variable.parameter" to "variable parameter" to have separated CSS classes
                )
                .as_bytes(),
            );
        };
        callback
    }

    /// Given a code content dynamically load this Tree-sitter parser return HTML
    /// based on the highlighted tokens of your code.
    /// If the highlight fails, it returns the code without modification.
    pub fn highlight(&self, code: &str) -> Html {
        let mut renderer = HtmlRenderer::new();
        match Highlighter::new()
            .highlight(self.highlight_config, code.as_bytes(), None, |_| None)
            .and_then(|highlights| {
                renderer.render(
                    highlights,
                    code.as_bytes(),
                    &self.get_callback_to_apply_highlight_on_token(),
                )
            }) {
            Ok(_) => Html(
                String::from_utf8(renderer.html)
                    .unwrap_or("Rendered HTML is not a valid UTF8, could not render.".to_string()),
            ),
            Err(_) => Html(code.to_string()),
        }
    }

    /// Normalise code block given lang to a set of known equivalence
    /// like js -> javascript, vuejs -> vue
    pub fn normalize_lang(given: &String) -> String {
        let given = given.as_str();
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
            _ => given,
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::{env::current_dir, fs::create_dir_all, path::PathBuf};

    use tree_sitter_loader::Loader;

    use crate::preview::{
        preview::Html,
        tree_sitter_grammars::{
            get_test_grammar_repos, get_unique_local_tree_sitter_grammars_folder,
            TreeSitterGrammarsManager, TEST_GRAMMAR,
        },
    };

    use super::TreeSitterHighlighter;

    #[test]
    #[ignore = "Network dependency"]
    fn test_highlight_with_test_grammar() {
        let mut m = TreeSitterGrammarsManager::new_with_grammars_folder(
            get_unique_local_tree_sitter_grammars_folder(),
        )
        .unwrap();
        m.install(&get_test_grammar_repos()).unwrap();

        let snippet = "color: blue";
        let mut loader = Loader::new().unwrap();
        let h = TreeSitterHighlighter::new(&mut loader, &TEST_GRAMMAR.to_string(), &m).unwrap();
        assert_eq!(h.highlight(snippet), Html("<span class='tag'>color</span><span class='punctuation delimiter'>:</span> <span class='attribute'>blue</span>\n".to_string()));

        let snippet = "#form { border: 1px solid #55232; }";
        assert_eq!(h.highlight(snippet), Html("<span class='punctuation delimiter'>#</span><span class='property'>form</span> <span class='punctuation bracket'>{</span> <span class='property'>border</span><span class='punctuation delimiter'>:</span> <span class='number'>1<span class='type'>px</span></span> solid <span class='string special'><span class='punctuation delimiter'>#</span>55232</span><span class='punctuation delimiter'>;</span> <span class='punctuation bracket'>}</span>\n".to_string()));
    }
}
