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

    /// Install a new grammar from a given git HTTPS URL
    pub fn install(&mut self, git_repo_https_url: &'a str) -> Result<(), String> {
        // Only clone the repository if it is not already present
        // Note: we consider 2 repositories with the name folder name to be equivalent for now
        let repos_name =
            GitRepos::validate_and_extract_repos_name_from_https_url(git_repo_https_url)?;
        let repos =
            match GitRepos::from_existing_folder(&TREE_SITTER_GRAMMARS_FOLDER.join(&repos_name)) {
                Ok(repos) => repos,
                Err(_) => GitRepos::from_clone(git_repo_https_url, &TREE_SITTER_GRAMMARS_FOLDER)?,
            };

        self.loader.force_rebuild(true);
        self.loader
            .compile_parser_at_path(
                repos.path(),
                repos.path().clone().join(repos_name + ".so"),
                // TODO: this
                // will only work on Linux...
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
    pub fn list_installed_langs(&mut self) -> Result<Vec<String>, String> {
        let mut config = Config::default();
        config
            .parser_directories
            .push(TREE_SITTER_GRAMMARS_FOLDER.clone());
        self.loader
            .find_all_languages(&config)
            .map_err(|e| e.to_string())?;

        Ok(self
            .loader
            .get_all_language_configurations()
            .iter()
            .map(|lc| lc.0.language_name.clone())
            .collect())
    }

    /// Make sure local dependencies are installed, such as a GCC and git
    /// Returns Ok if all good or an Err with a reason
    pub fn check_local_deps() -> Result<(), String> {
        Command::new("gcc")
            .arg("--version")
            .output()
            .map_err(|_| "Gcc not installed or not available in PATH".to_string())?;

        if !GitRepos::is_git_installed() {
            return Err("Git is not installed".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{env::current_dir, fs::create_dir_all, path::PathBuf};

    use crate::preview::{
        proposed_grammars::PROPOSED_GRAMMAR_SOURCES,
        tree_sitter_grammars::TreeSitterGrammarsManager,
    };

    fn get_test_grammar_repos() -> String {
        PROPOSED_GRAMMAR_SOURCES.get("css").unwrap().to_string()
    }

    fn get_unique_local_tree_sitter_grammars_folder() -> PathBuf {
        let base = current_dir()
            .unwrap()
            .join("target")
            .join("tree-sitter-grammars");
        let random: u32 = rand::random_range(0..=1000000000);
        let unique_folder = base.join(random.to_string());
        if !unique_folder.exists() {
            create_dir_all(&unique_folder).expect("Couldn't create tests folder inside target/");
        }
        unique_folder
    }

    #[test]
    fn test_can_install_csv_grammar() {
        // Configure another grammars installation folder to avoid impacting the dev environnement
        std::env::set_var(
            "TREE_SITTER_GRAMMARS_FOLDER",
            get_unique_local_tree_sitter_grammars_folder(),
        );
        let mut m = TreeSitterGrammarsManager::new().unwrap();
        assert!(m.list_installed_langs().unwrap().is_empty());
        let result = m.install(&get_test_grammar_repos());
        dbg!(&result);
        result.unwrap();
    }
    #[test]
    fn test_check_local_deps() {
        assert!(TreeSitterGrammarsManager::check_local_deps().is_ok());

        std::env::set_var("PATH", ""); // empty the PATH so git will not be found
        assert!(TreeSitterGrammarsManager::check_local_deps().is_err());
    }
}
