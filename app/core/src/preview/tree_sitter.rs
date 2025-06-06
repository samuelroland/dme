// Tree-sitter highlighter via external grammars on disk
// This implementation has been inspired/helped by this article
// https://dotat.at/@/2025-03-30-hilite.html
// and the associated implementation
// https://dotat.at/cgi/git/wwwdotat.git/blob/HEAD:/src/hilite.rs

use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    collections::HashSet,
    fs::{self, read_to_string, File},
    os::fd::{AsFd, AsRawFd},
    path::{Path, PathBuf},
    process::{exit, Command},
    str::FromStr,
};
use tree_sitter_highlight::{Highlight, HighlightConfiguration, Highlighter, HtmlRenderer};
use tree_sitter_loader::{CompileConfig, Loader};

use crate::util::git::GitRepos;

use super::preview::Html;

// TODO: refactor with dynamic path from configuration
const HIGHLIGHT_QUERIES_PATH: &str = "queries/highlights.scm";

/// A highlighter for a specific language, once loaded it can highlight multiple code snippets of
/// the same programming language
pub(crate) struct TreeSitterHighlighter<'a> {
    lang: &'a str,
}

impl<'a> TreeSitterHighlighter<'a> {
    /// Try to create a new highlighter based
    pub fn new(lang: &'a str) -> Result<Self, String> {
        if Self::check_lang_grammar_installed(lang) {
            Ok(TreeSitterHighlighter { lang })
        } else {
            Err("The grammar {lang} is not installed locally".to_string())
        }
    }

    /// Just get the language of the highlighter defined via new()
    pub fn get_lang(&self) -> &'a str {
        self.lang
    }

    fn check_lang_grammar_installed(lang: &'a str) -> bool {
        false
    }

    fn parse_highlighing_queries() -> Vec<String> {
        // 2 sub directories, maybe should be taken from
        let parser_directory = grammar_directory.join("src");
        let highlight_queries_content =
            read_to_string(grammar_directory.join(HIGHLIGHT_QUERIES_PATH)).unwrap();

        let mut highlight_names_in_queries: HashSet<&str> = HashSet::new();
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\@[A-Za-z\.]+").unwrap());
        RE.find_iter(&highlight_queries_content).for_each(|n| {
            highlight_names_in_queries.insert(&n.as_str()[1..]);
        });
        highlight_names_in_queries
            .into_iter()
            .collect::<Vec<String>>()
    }

    /// Given a code content + a Tree-sitter grammar directory for this language
    /// dynamically load this Tree-sitter parser and render HTML back
    /// It will detect all highlight names present in the queries files
    pub fn highlight(&self, code: &str) -> Html {
        let loader = Loader::new();
        let lang = loader
            .unwrap()
            .load_language_at_path(CompileConfig::new(&parser_directory, None, None))
            .unwrap();
        let name = lang.name().unwrap();

        println!("Loaded lang {name} !!");
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang).unwrap();
        println!("Parsing content of {code}");
        let tree = parser.parse(code, None).unwrap();
        println!("Got tree {tree:?} !!");

        // Create a highlighter that do recognize all detected highlight names
        let mut highlighter = Highlighter::new();
        let mut highlight_config =
            HighlightConfiguration::new(lang, name, &highlight_queries_content, "", "").unwrap();
        let recognized_highlights_name = self::parse_highlighing_queries();
        highlight_config.configure(recognized_highlights_name);

        // Do the final highlighting of given code
        let highlights = highlighter
            .highlight(&highlight_config, code.as_bytes(), None, |_| None)
            .unwrap();

        let mut renderer = HtmlRenderer::new();
        let callback = |highlight: Highlight, output: &mut Vec<u8>| {
            output.extend_from_slice(
                format!(
                    "class='{}'",
                    recognized_highlights_name
                        .get(highlight.0)
                        .unwrap()
                        .replace(".", " ")
                )
                .as_bytes(),
            );
        };

        renderer
            .render(highlights, code.as_bytes(), &callback)
            .unwrap();
        let html = String::from_utf8(renderer.html).unwrap();
        println!("Generated html:\n{}", html);
        Html(html)
    }

    /// Normalise code block given lang to a set of known equivalence
    /// like js -> javascript, vuejs -> vue
    fn normalize_lang(given: &'a str) -> &'a str {
        match given {
            "bash" | "sh" | "shell" => "bash",
            "js" => "javascript",
            "vuejs" => "vue",
            "py" => "python",
            "md" => "markdown",
            "hs" => "haskell",
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {}

/// Manager of local Tree-Sitter grammars
/// Make it easy to download, compile, list, remove grammars
pub(crate) struct TreeSitterGrammarsManager {
    loader: Loader,
}

impl<'a> TreeSitterGrammarsManager {
    fn install(git_repo_url: &'a str) -> Result<(), String> {
        let loader = Loader::new();
        GitRepos::from_clone(git_repo_url)?;
        loader.compile_parser_at_path(grammar_path, output_path, flags)
    }

    fn remove(lang: &'a str) -> Result<(), String> {
        todo!()
    }

    fn proposed_grammar_sources() -> Vec<(String, String)> {
        Vec::from(PROPOSED_GRAMMAR_SOURCES)
            .iter()
            .map(|g| (g.0.to_string(), g.1.to_string()))
            .collect()
    }

    /// Retrieve a list of languages accessible by Tree-Sitter
    fn list_installed_langs(&self) -> Result<Vec<String>, String> {
        Ok(self
            .loader
            .get_all_language_configurations()
            .iter()
            .map(|lc| lc.0.language_name.clone())
            .collect::<Vec<String>>())
    }

    // Make sure local dependencies are installed, such as a GCC and git
    fn check_local_deps(git_repo_url: &'a str) -> Result<(), String> {
        todo!()
    }
}

/// This is shown by the UI as proposed default links from the tree-sitter and tree-sitter-grammars Github organisations
const PROPOSED_GRAMMAR_SOURCES: [(&str, &str); 22] = [
    (
        "yaml",
        "https://github.com/tree-sitter-grammars/tree-sitter-yaml",
    ),
    (
        "lua",
        "https://github.com/tree-sitter-grammars/tree-sitter-lua",
    ),
    (
        "make",
        "https://github.com/tree-sitter-grammars/tree-sitter-make",
    ),
    (
        "toml",
        "https://github.com/tree-sitter-grammars/tree-sitter-toml",
    ),
    (
        "vue",
        "https://github.com/tree-sitter-grammars/tree-sitter-vue",
    ),
    (
        "csv",
        "https://github.com/tree-sitter-grammars/tree-sitter-csv",
    ),
    (
        "xml",
        "https://github.com/tree-sitter-grammars/tree-sitter-xml",
    ),
    ("cpp", "https://github.com/tree-sitter/tree-sitter-cpp"),
    ("php", "https://github.com/tree-sitter/tree-sitter-php"),
    ("rust", "https://github.com/tree-sitter/tree-sitter-rust"),
    ("scala", "https://github.com/tree-sitter/tree-sitter-scala"),
    ("css", "https://github.com/tree-sitter/tree-sitter-css"),
    ("regex", "https://github.com/tree-sitter/tree-sitter-regex"),
    ("html", "https://github.com/tree-sitter/tree-sitter-html"),
    ("java", "https://github.com/tree-sitter/tree-sitter-java"),
    ("bash", "https://github.com/tree-sitter/tree-sitter-bash"),
    (
        "typescript",
        "https://github.com/tree-sitter/tree-sitter-typescript",
    ),
    ("json", "https://github.com/tree-sitter/tree-sitter-json"),
    ("go", "https://github.com/tree-sitter/tree-sitter-go"),
    (
        "haskell",
        "https://github.com/tree-sitter/tree-sitter-haskell",
    ),
    ("c", "https://github.com/tree-sitter/tree-sitter-c"),
    (
        "javascript",
        "https://github.com/tree-sitter/tree-sitter-javascript",
    ),
];
