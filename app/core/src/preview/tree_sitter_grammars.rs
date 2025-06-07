use std::{
    collections::HashMap,
    fs::create_dir,
    path::{Path, PathBuf},
};

use etcetera::{AppStrategy, AppStrategyArgs};
use once_cell::sync::Lazy;
use tree_sitter::Language;
use tree_sitter_loader::{CompileConfig, Loader};

use crate::util::git::GitRepos;

/// Use the DATA HOME strategy to determine the base folder grammars and cloned and managed
/// on Linux it will be under ~/.local/share/tree-sitter-grammars
static TREE_SITTER_GRAMMARS_FOLDER: Lazy<PathBuf> = Lazy::new(|| {
    let folder = etcetera::choose_app_strategy(AppStrategyArgs {
        app_name: "tree-sitter-grammars".to_string(),
        ..Default::default()
    })
    .unwrap()
    .data_dir()
    .to_path_buf();
    if !folder.exists() {
        let _ = create_dir(&folder);
    }
    folder
});

/// Manager of local Tree-Sitter grammars
/// Make it easy to download, compile, list, remove grammars
pub struct TreeSitterGrammarsManager {
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
    pub(crate) fn get_repos_for_lang(lang: &str) -> Result<GitRepos, String> {
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
