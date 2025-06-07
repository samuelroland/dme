#[cfg(test)]
use pretty_assertions::{assert_eq, assert_ne};

use std::{
    env::current_dir,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::LazyLock,
};

use regex::Regex;

/// This file implement a nice and easy interface to do simple Git operations
/// on a given Git repository
pub struct GitRepos {
    path: PathBuf,
}

static GIT_CLONE_HTTPS_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Note: the +? after repos name is important to be non greedy and not take a possible .git
    // afterwards so the group 4 can be used as a way to extract the name
    Regex::new(r#"^https://(([a-z0-9-]+\.)+[a-z0-9-]+)/([a-z0-9-]+)/([a-z0-9-_.]+?)(\.git)?$"#)
        .expect("GIT_CLONE_HTTPS_LINK_REGEX failed to compile")
});

impl GitRepos {
    /// Just extract the path of the repository
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get a git repository from an existing folder on disk
    pub fn from_existing_folder(directory: &PathBuf) -> Result<Self, String> {
        if !directory.exists() {
            return Err(format!(
                "The folder {:?} doesn't exist, cannot use as a git repository.",
                directory
            ));
        }

        if !directory.join(".git").exists() {
            return Err(format!(
                "The folder {:?} exist but is not a git repository.",
                directory
            ));
        }

        Ok(GitRepos {
            path: directory.clone(),
        })
    }

    /// Get a git repository after cloning it, make sure the link is valid before hand
    pub fn from_clone(git_clone_url: &str, base_directory: &PathBuf) -> Result<Self, String> {
        let output = Self::run_git_cmd(&vec!["clone", git_clone_url], base_directory)?;
        let grammar_folder_name =
            Self::validate_and_extract_repos_name_from_https_url(git_clone_url)?;
        if output.status.success() {
            Ok(GitRepos {
                path: base_directory.join(grammar_folder_name),
            })
        } else {
            Err(format!("Failed to git clone {}", git_clone_url).to_string())
        }
    }

    /// Try to pull a repository, only if is remote
    pub fn pull(&self) -> Result<bool, String> {
        if self.is_remote().is_ok_and(|v| v) {
            Err(format!("Cannot pull a local only repository on {:?}", self.path).to_string())
        } else {
            let output = Self::run_git_cmd(&vec!["pull"], &self.path)?;
            Ok(output.status.success())
        }
    }

    /// Check if the repository is a remote repository by checking if
    /// a remote.origin.url config entry exists
    pub fn is_remote(&self) -> Result<bool, String> {
        let output = Self::run_git_cmd(&vec!["config", "remote.origin.url"], &self.path)?;
        Ok(output.status.success())
    }

    /// Given a git clone link like "https://codeberg.org/samuelroland/productivity",
    /// make sure the link is valid and extract the name "productivity"
    fn validate_and_extract_repos_name_from_https_url(url: &str) -> Result<String, String> {
        Ok(GIT_CLONE_HTTPS_LINK_REGEX
            .captures(url)
            .ok_or_else(|| "Given URL not a valid HTTPS git clone URL".to_string())?
            .get(4)
            .expect("Error, no group 4 in regex for GIT_CLONE_HTTPS_LINK_REGEX")
            .as_str()
            .to_string())
    }

    /// Run a git commands with given args and base_directory in which the command will be ran
    fn run_git_cmd(args: &Vec<&str>, base_directory: &PathBuf) -> Result<Output, String> {
        let cmd = Command::new("git")
            .args(args)
            .current_dir(base_directory)
            .output();

        cmd.map_err(|e| format!("Failed to run git {}: {e}", args.join(" ")))
    }

    // Return true if Git is installed
    pub fn is_git_installed() -> bool {
        if let Ok(output) = Self::run_git_cmd(
            &vec!["--version"],
            &current_dir().expect("Couldn't get current directory to run git --version"),
        ) {
            output.status.success()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env::current_dir,
        fs::{create_dir, remove_dir_all},
        path::PathBuf,
    };

    use crate::util::git::GitRepos;

    fn get_tests_folder() -> PathBuf {
        current_dir().unwrap().join("target").join("tests")
    }

    fn get_fresh_tests_folder() -> PathBuf {
        let folder = get_tests_folder();
        if folder.exists() {
            remove_dir_all(&folder).expect("Couldn't remove existing tests directory");
        }
        create_dir(&folder).expect("Couldn't create tests folder inside target/");
        folder
    }

    #[test]
    fn test_from_clone_with_invalid_link() {
        assert!(GitRepos::from_clone("not a valid URL", &get_tests_folder()).is_err());
    }

    #[test]
    fn test_from_clone_with_valid_link() {
        let tests_folder = &get_fresh_tests_folder();
        let repos =
            GitRepos::from_clone("https://github.com/samuelroland/cloneme.git", tests_folder)
                .unwrap();

        assert_eq!(tests_folder.join("cloneme"), *repos.path());

        assert!(repos.path().exists());
    }

    #[test]
    fn test_extract_repos_name_from_https_url() {
        assert_eq!(
            GitRepos::validate_and_extract_repos_name_from_https_url(
                "https://codeberg.org/samuelroland/productivity"
            )
            .unwrap(),
            "productivity".to_string()
        );
        assert_eq!(
            GitRepos::validate_and_extract_repos_name_from_https_url(
                "https://github.com/tree-sitter/tree-sitter-rust.git"
            )
            .unwrap(),
            "tree-sitter-rust".to_string()
        );
        assert!(GitRepos::validate_and_extract_repos_name_from_https_url(
            "git@github.com:samuelroland/cloneme.git" // valid url but not https form
        )
        .is_err());
        assert!(GitRepos::validate_and_extract_repos_name_from_https_url(
            "https://github.com/tree-sitter"
        )
        .is_err());
        assert!(GitRepos::validate_and_extract_repos_name_from_https_url("blabl").is_err());
        assert!(GitRepos::validate_and_extract_repos_name_from_https_url(
            "https://github.com/tree-sitter/tree-sitter-rust.git$243536"
        )
        .is_err());
    }

    #[test]
    fn test_is_git_installed() {
        // this would fail on a machine without Git, this is good as we need it for further testing
        assert!(GitRepos::is_git_installed());

        std::env::set_var("PATH", ""); // empty the PATH so git will not be found
        assert!(!GitRepos::is_git_installed());
    }
}
