use std::{
    collections::{BTreeSet, HashSet},
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

/// Manager of local Tree-Sitter grammars, cloned with Git from any Git HTTPS links
/// We also have a list of official grammars on GitHub for ~22 languages in `proposed_grammars.rs`
/// This manager makes it very easy to install grammars, find their local folder, load, update, or remove them
pub struct TreeSitterGrammarsManager {
    loader: Loader,
    /// The final grammars folder, can be the DEFAULT_TREE_SITTER_GRAMMARS_FOLDER
    /// or another one defined in new()
    final_grammars_folder: PathBuf,
}

impl TreeSitterGrammarsManager {
    /// Create a new manager with a loader that needs a Tree-Sitter LIBDIR
    pub fn new() -> Result<Self, String> {
        let loader = Loader::new().map_err(|e| e.to_string())?;

        /// Use the DATA HOME strategy to determine the base folder grammars and cloned and managed
        /// on Linux it will be under ~/.local/share/tree-sitter-grammars
        static DEFAULT_TREE_SITTER_GRAMMARS_FOLDER: Lazy<PathBuf> = Lazy::new(|| {
            let folder = etcetera::choose_app_strategy(AppStrategyArgs {
                app_name: "tree-sitter-grammars".to_string(),
                ..Default::default()
            })
            .unwrap() // only in case it couldn't determine the home the directory,
            // in this case we don't know where to put these grammars and the only option is to panic
            .data_dir()
            .to_path_buf();
            if !folder.exists() {
                let _ = create_dir(&folder);
            }
            folder
        });
        Ok(TreeSitterGrammarsManager {
            loader,
            final_grammars_folder: DEFAULT_TREE_SITTER_GRAMMARS_FOLDER.clone(),
        })
    }

    /// Create a manager by specifying another folder instead of DEFAULT_TREE_SITTER_GRAMMARS_FOLDER
    /// Public only for this crate because only useful for testing
    pub(crate) fn new_with_grammars_folder(
        another_grammars_folder: PathBuf,
    ) -> Result<Self, String> {
        let loader = Loader::new().map_err(|e| e.to_string())?;
        Ok(TreeSitterGrammarsManager {
            loader,
            final_grammars_folder: another_grammars_folder,
        })
    }

    /// Install a new grammar from a given git HTTPS URL
    pub fn install(&mut self, git_repo_https_url: &str) -> Result<(), String> {
        // Only clone the repository if it is not already present
        // Note: we consider 2 repositories with the name folder name to be equivalent for now
        let repos_name =
            GitRepos::validate_and_extract_repos_name_from_https_url(git_repo_https_url)?;
        let repos =
            match GitRepos::from_existing_folder(&self.final_grammars_folder.join(&repos_name)) {
                Ok(repos) => repos,
                Err(_) => GitRepos::from_clone(git_repo_https_url, &self.final_grammars_folder)?,
            };

        self.loader.force_rebuild(true);
        self.compile_at_path(repos.path())
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Update the grammar behind the given lang and returns true if the grammar has changed
    pub fn update(&mut self, lang: &str) -> Result<bool, String> {
        let repos = self.get_repos_for_lang(lang)?;
        let pulled_something = repos.pull()?;
        // Only recompile if we pulled something
        if pulled_something {
            self.compile_at_path(repos.path())?;
        }
        Ok(pulled_something)
    }

    /// Delete the grammar behind the given lang
    /// This is consuming self to avoid reusing it after deletion
    pub fn delete(&mut self, lang: &str) -> Result<(), String> {
        let repos = self.get_repos_for_lang(lang)?;
        let result = std::fs::remove_dir_all(repos.path())
            .map(|_| ())
            .map_err(|e| e.to_string());

        // Reset the loader as that's the only way to clean the internal list of grammars
        self.loader = Loader::new().map_err(|e| e.to_string())?;
        result
    }

    /// Helper to quickly get the repository behind the lang
    pub(crate) fn get_repos_for_lang(&self, lang: &str) -> Result<GitRepos, String> {
        GitRepos::from_existing_folder(
            &self
                .final_grammars_folder
                .join(format!("tree-sitter-{}", lang)),
        )
    }

    /// This is a replacement over the Loader::compile_parser_at_path() method
    /// because it forces us to decide on the output file. As the shared library
    /// extension is different on the 3 main OS, that's better to let it manage this complexity
    fn compile_at_path(&mut self, repos_path: &Path) -> Result<Language, String> {
        let src_path = repos_path.join("src");
        // No output path, let it take the default in TREE_SITTER_LIBDIR
        let config = CompileConfig::new(&src_path, None, None);
        self.loader.force_rebuild(true); // this doesn't build otherwise
        self.loader
            .load_language_at_path(config)
            .map_err(|e| e.to_string())
    }

    /// Retrieve a list of languages accessible by Tree-Sitter
    pub fn list_installed_langs(&mut self) -> Result<Vec<String>, String> {
        // Create our own config.json in memory only with only
        // self.final_grammars_folder as directory for grammars
        let mut config = Config::default();
        config
            .parser_directories
            .push(self.final_grammars_folder.clone());
        // This is necessary to refresh the list inside the loader
        self.loader
            .find_all_languages(&config)
            .map_err(|e| e.to_string())?;

        // As the get_all_language_configurations is finding duplicated folders for some reason
        // I don't really understand the implementation sens of the approach
        // but we are forced to make this list unique,
        // by collecting results into a Set before going back to a Vec
        let unique_langs = self
            .loader
            .get_all_language_configurations()
            .iter()
            .map(|lc| lc.0.language_name.to_owned())
            .collect::<BTreeSet<_>>()
            .iter()
            .cloned()
            .collect::<Vec<String>>();

        Ok(unique_langs)
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
pub static TEST_GRAMMAR: &str = "css";
#[cfg(test)]
pub fn get_test_grammar_repos() -> String {
    use super::proposed_grammars::PROPOSED_GRAMMAR_SOURCES;

    PROPOSED_GRAMMAR_SOURCES
        .get(TEST_GRAMMAR)
        .unwrap()
        .to_string()
}
#[cfg(test)]
pub fn get_unique_local_tree_sitter_grammars_folder() -> PathBuf {
    use std::{env::current_dir, fs::create_dir_all};

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
#[cfg(test)]
mod tests {
    use crate::preview::tree_sitter_grammars::get_test_grammar_repos;
    use crate::preview::tree_sitter_grammars::get_unique_local_tree_sitter_grammars_folder;
    use crate::preview::tree_sitter_grammars::TEST_GRAMMAR;

    use crate::{preview::tree_sitter_grammars::TreeSitterGrammarsManager, util::git::GitRepos};

    #[test]
    #[ignore = "Slow and network usage"]
    fn test_can_install_test_grammar() {
        // Configure another grammars installation folder to avoid impacting the dev environnement
        let grammars_folder = get_unique_local_tree_sitter_grammars_folder();
        let mut m = TreeSitterGrammarsManager::new_with_grammars_folder(grammars_folder).unwrap();
        assert!(m.list_installed_langs().unwrap().is_empty());
        let result = m.install(&get_test_grammar_repos());
        result.unwrap();
        assert_eq!(m.list_installed_langs().unwrap().len(), 1);
    }

    #[test]
    #[ignore = "Slow and network usage"]
    fn test_can_update_test_grammar() {
        let grammars_folder = get_unique_local_tree_sitter_grammars_folder();
        let mut m = TreeSitterGrammarsManager::new_with_grammars_folder(grammars_folder).unwrap();
        m.install(&get_test_grammar_repos()).unwrap();
        let has_been_updated = m.update(TEST_GRAMMAR).unwrap();
        assert!(!has_been_updated);

        GitRepos::run_git_cmd(
            &vec!["reset", "--hard", "HEAD~2"],
            m.get_repos_for_lang(TEST_GRAMMAR).unwrap().path(),
        )
        .unwrap();
        assert_eq!(m.list_installed_langs().unwrap().len(), 1);

        let has_been_updated = m.update(TEST_GRAMMAR).unwrap();
        assert!(has_been_updated);
    }

    #[test]
    #[ignore = "Slow and network usage"]
    fn test_can_remove_test_grammar() {
        let grammars_folder = get_unique_local_tree_sitter_grammars_folder();
        let mut m = TreeSitterGrammarsManager::new_with_grammars_folder(grammars_folder).unwrap();
        m.install(&get_test_grammar_repos()).unwrap();

        assert_eq!(m.list_installed_langs().unwrap().len(), 1);

        m.delete(TEST_GRAMMAR).unwrap();
        assert!(m.list_installed_langs().unwrap().is_empty());
    }

    #[test]
    fn test_check_local_deps() {
        let result = TreeSitterGrammarsManager::check_local_deps();
        dbg!(&result);
        assert!(result.is_ok());
        // Note: that's impossible to test without changing the PATH which affects other tests
        // assert!(TreeSitterGrammarsManager::check_local_deps().is_err());
    }
}
