// Tree-sitter highlighter via external grammars on disk
// This implementation has been inspired/helped by this article
// https://dotat.at/@/2025-03-30-hilite.html
// and the associated implementation
// https://dotat.at/cgi/git/wwwdotat.git/blob/HEAD:/src/hilite.rs

use etcetera::{AppStrategy, AppStrategyArgs};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, read_to_string, File},
    hash::Hash,
    iter::Map,
    os::fd::{AsFd, AsRawFd},
    path::{Path, PathBuf},
    process::{exit, Command},
    str::FromStr,
};
use tree_sitter::{Language, Parser};
use tree_sitter_highlight::{Highlight, HighlightConfiguration, Highlighter, HtmlRenderer};
use tree_sitter_loader::{CompileConfig, Config, Loader, PackageJSON};

use crate::util::git::GitRepos;

use super::preview::Html;

// TODO: refactor with dynamic path from configuration
const HIGHLIGHT_QUERIES_PATH: &str = "queries/highlights.scm";

static HIGHLIGHT_NAMES_PARSER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\@[A-Za-z\.]+").unwrap());

/// Use the DATA HOME strategy to determine the base folder grammars and cloned and managed
/// on Linux it will be under ~/.local/share/tree-sitter-grammars
static TREE_SITTER_GRAMMARS_FOLDER: Lazy<PathBuf> = Lazy::new(|| {
    etcetera::choose_app_strategy(AppStrategyArgs {
        app_name: "tree-sitter-grammars".to_string(),
        ..Default::default()
    })
    .unwrap()
    .data_dir()
    .to_path_buf()
});

/// A highlighter for a specific language, once loaded it can highlight multiple code snippets of
/// the same programming language
pub(crate) struct TreeSitterHighlighter<'a> {
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

/// Manager of local Tree-Sitter grammars
/// Make it easy to download, compile, list, remove grammars
pub(crate) struct TreeSitterGrammarsManager {
    loader: Loader,
}

impl<'a> TreeSitterGrammarsManager {
    /// Create a new manager with a loader that needs a Tree-Sitter LIBDIR
    pub fn new() -> Result<Self, String> {
        let loader = Loader::new().map_err(|e| e.to_string())?;
        Ok(TreeSitterGrammarsManager { loader })
    }

    /// Install a new grammar from a given git link
    pub fn install(&self, git_repo_url: &'a str) -> Result<(), String> {
        let repos = GitRepos::from_clone(git_repo_url, &TREE_SITTER_GRAMMARS_FOLDER)?;
        self.loader
            .compile_parser_at_path(
                repos.path(),
                repos.path().clone(),
                Vec::default().as_slice(),
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Update the grammar behind the given lang
    pub fn update(&self, lang: &'a str) -> Result<(), String> {
        let repos = Self::get_repos_for_lang(lang)?;
        repos.pull()?;
        self.compile_at_path(repos.path())?;
        Ok(())
    }

    /// Delete the grammar behind the given lang
    /// This is consuming self to avoid reusing it after deletion
    pub fn delete(self, lang: &'a str) -> Result<(), String> {
        let repos = Self::get_repos_for_lang(lang)?;
        std::fs::remove_dir_all(repos.path())
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Helper to quickly get the repository behind the lang
    fn get_repos_for_lang(lang: &str) -> Result<GitRepos, String> {
        GitRepos::from_existing_folder(
            &TREE_SITTER_GRAMMARS_FOLDER.join(format!("tree-sitter-{}", lang)),
        )
    }

    fn compile_at_path(&self, repos_path: &Path) -> Result<Language, String> {
        let src_path = repos_path.join("src");
        // No output path, let it take the default in TREE_SITTER_LIBDIR
        let config = CompileConfig::new(&src_path, None, None);
        self.loader
            .load_language_at_path(config)
            .map_err(|e| e.to_string())

        // Note: Do not use loader.compile_parser_at_path because it forces us to
        // decide on the output file, we is not trivial to generate
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
pub static PROPOSED_GRAMMAR_SOURCES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
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
    ])
});
