use std::{
    fs::create_dir,
    path::{Path, PathBuf},
    process::Command,
};

use etcetera::{AppStrategy, AppStrategyArgs};
use once_cell::sync::Lazy;
use pretty_assertions::{assert_eq, assert_ne};
use tree_sitter::Language;
use tree_sitter_loader::{CompileConfig, Config, Loader};

use crate::util::git::{self, GitRepos};

/// Use the DATA HOME strategy to determine the base folder grammars and cloned and managed
/// on Linux it will be under ~/.local/share/tree-sitter-grammars
static TREE_SITTER_GRAMMARS_FOLDER: Lazy<PathBuf> = Lazy::new(|| {
    let folder = match std::env::var("TREE_SITTER_GRAMMARS_FOLDER") {
        Ok(folder) => PathBuf::from(folder),
        Err(_) => etcetera::choose_app_strategy(AppStrategyArgs {
            app_name: "tree-sitter-grammars".to_string(),
            ..Default::default()
        })
        .unwrap() // only in case it couldn't determine the home the directory,
        // in this case we don't know where to put these grammars and the only option is to panic
        .data_dir()
        .to_path_buf(),
    };
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
