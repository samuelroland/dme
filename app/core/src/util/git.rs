#[cfg(test)]
use pretty_assertions::{assert_eq, assert_ne};

use std::{
    env::current_dir,
    io::Read,
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
    /// We can define the --depth via only_latest_commits and the --single-branch argument for
    /// faster clones when we don't need the full history or all the branches
    pub fn from_clone(
        git_clone_url: &str,
        base_directory: &PathBuf,
        only_latest_commits: Option<usize>,
        single_branch: bool,
    ) -> Result<Self, String> {
        let mut args: Vec<String> = vec!["clone".to_owned(), git_clone_url.to_owned()];
        let count_str = "";
        if let Some(count) = only_latest_commits {
            args.push("--depth".to_string());
            args.push(count.to_string())
        }
        if single_branch {
            args.push("--single-branch".to_owned());
        }
        // Note: we are forced to create a Vec<&str> afterwards because count.to_string() need to
        // be owned first to exist long enough
        let args_ref = args.iter().map(|e| e.as_ref()).collect();
        let output = Self::run_git_cmd(&args_ref, base_directory)?;
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

    /// Try to pull a repository, only if is remote and return
    /// true if some commits were pulled, false if it was already up-to-date
    pub fn pull(&self) -> Result<bool, String> {
        if self.is_remote().is_ok_and(|v| v) {
            let hash_before = self.get_last_commit_hash()?;
            Self::run_git_cmd(&vec!["pull"], &self.path)?;
            let hash_after = self.get_last_commit_hash()?;
            Ok(hash_before != hash_after)
        } else {
            Err(format!("Cannot pull a local only repository on {:?}", self.path).to_string())
        }
    }

    /// Get last commit hash by running: git rev-parse HEAD
    fn get_last_commit_hash(&self) -> Result<String, String> {
        let output = Self::run_git_cmd(&vec!["rev-parse", "HEAD"], &self.path)?;
        Ok(String::from_utf8(output.stdout)
            .map_err(|e| e.to_string())?
            .trim()
            .to_string())
    }

    /// Check if the repository is a remote repository by checking if
    /// a remote.origin.url config entry exists
    pub fn is_remote(&self) -> Result<bool, String> {
        let output = Self::run_git_cmd(&vec!["config", "--get", "remote.origin.url"], &self.path)?;
        Ok(output.status.success())
    }

    /// Given a git clone link like "https://codeberg.org/samuelroland/productivity",
    /// make sure the link is valid and extract the name "productivity"
    pub fn validate_and_extract_repos_name_from_https_url(url: &str) -> Result<String, String> {
        Ok(GIT_CLONE_HTTPS_LINK_REGEX
            .captures(url)
            .ok_or_else(|| "Given URL not a valid HTTPS git clone URL".to_string())?
            .get(4)
            .expect("Error, no group 4 in regex for GIT_CLONE_HTTPS_LINK_REGEX")
            .as_str()
            .to_string())
    }

    /// Run a git commands with given args and exec_directory in which the command will be ran
    pub(crate) fn run_git_cmd(
        args: &Vec<&str>,
        exec_directory: &PathBuf,
    ) -> Result<Output, String> {
        let cmd = Command::new("git")
            .args(args)
            .current_dir(exec_directory)
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
        fs::{create_dir, create_dir_all, remove_dir_all},
        path::PathBuf,
        thread::{self, sleep},
        time::{Duration, Instant},
    };
    // Note: I'm using a public Git repos almost empty to tests git clone and git pull operatiosn
    const REAL_GIT_REPO: &str = "https://github.com/samuelroland/cloneme.git";

    use crate::util::git::GitRepos;

    fn get_unique_tests_subfolder() -> PathBuf {
        let base = current_dir().unwrap().join("target").join("tests");
        let random: u32 = rand::random_range(0..=1000000000);
        let unique_folder = base.join(random.to_string());
        if !unique_folder.exists() {
            create_dir_all(&unique_folder).expect("Couldn't create tests folder inside target/");
        }
        unique_folder
    }

    #[test]
    fn test_from_clone_with_invalid_link() {
        assert!(GitRepos::from_clone(
            "not a valid URL",
            &get_unique_tests_subfolder(),
            None,
            false
        )
        .is_err());
    }

    #[test]
    #[ignore = "This test only work in serial mode and is slow"]
    fn test_from_clone_with_valid_link() {
        let tests_folder = &get_unique_tests_subfolder();
        let repos = GitRepos::from_clone(REAL_GIT_REPO, tests_folder, None, false).unwrap();

        assert_eq!(tests_folder.join("cloneme"), *repos.path());
        assert!(repos.path().exists());
        assert!(repos.is_remote().unwrap());
    }

    #[test]
    fn test_from_existing_folder_with_local_normal_folder_fails() {
        // No .git found
        assert!(GitRepos::from_existing_folder(&current_dir().unwrap().join("target")).is_err());
    }

    #[test]
    #[ignore = "This test only work in serial mode and is slow"]
    fn test_from_existing_folder_with_git_repos_works() {
        let tests_folder = &get_unique_tests_subfolder();
        let repos = GitRepos::from_clone(REAL_GIT_REPO, tests_folder, None, false).unwrap();

        let new_repos = GitRepos::from_existing_folder(&tests_folder.join("cloneme")).unwrap();
        assert!(new_repos.is_remote().unwrap());
    }

    #[test]
    #[ignore = "This test only work in serial mode and is slow"]
    fn test_clone_with_last_commit_one_branch_is_faster() {
        let tests_folder = &get_unique_tests_subfolder();

        let start = Instant::now();
        let repos = GitRepos::from_clone(REAL_GIT_REPO, tests_folder, None, false).unwrap();
        let duration1 = start.elapsed();

        let tests_folder = &get_unique_tests_subfolder();

        let start = Instant::now();
        let repos = GitRepos::from_clone(REAL_GIT_REPO, tests_folder, Some(1), true).unwrap();
        let duration2 = start.elapsed();

        assert!(
            duration2 < duration1,
            "Cloning a repository should be faster with a single branch + only the last commit"
        );
    }

    #[test]
    #[ignore = "This test only work in serial mode and is slow"]
    fn test_pull_works() {
        let tests_folder = &get_unique_tests_subfolder();
        let repos = GitRepos::from_clone(REAL_GIT_REPO, tests_folder, None, true).unwrap();
        assert!(repos.path().join("newfile").exists());
        assert!(!repos.pull().unwrap()); // nothing to pull !
                                         // Destroy latest commit with its changes to simulate a not update repos that needs to be pull
        GitRepos::run_git_cmd(&vec!["reset", "--hard", "HEAD~1"], repos.path()).unwrap();
        assert!(!repos.path().join("newfile").exists());
        assert!(repos.pull().unwrap()); // some commits were pull as it returns true
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

        // Note: that's impossible to test without changing the PATH which affects other tests
        // assert!(!GitRepos::is_git_installed());
    }
}
