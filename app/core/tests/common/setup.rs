// This contains functions used to setup large test data for integration tests and benchmarks

use std::ffi::OsStr;
use std::fmt::Write;
use std::fs::{create_dir_all, read_dir, read_to_string};
use std::path::{Path, PathBuf};

use dme_core::preview::proposed_grammars::PROPOSED_GRAMMAR_SOURCES;
use dme_core::util::git::GitRepos;

const MDN_GIT_REPOSITORY: &str = "https://github.com/mdn/content";

/// Clone the last commit of the MDN (Mozilla Developers Network) documentation
/// with over 13700 Markdown files (as of 2025-06-10)
pub fn clone_mdn_content() -> PathBuf {
    let path = PathBuf::from("target");
    if !path.exists() {
        GitRepos::from_clone(MDN_GIT_REPOSITORY, &path, Some(1), true).unwrap();
    }
    path.join("content")
}

const CODE_SNIPPETS_REPOS: &str = "https://github.com/TheRenegadeCoder/sample-programs.git";
const CODE_SNIPPETS_REPOS_DESTINATION: &str = "target/sample-programs";
const SUBFOLDER: &str = "target/all-grammars";
const OUTPUT_MD_PREFIX: &str = "target/large-";

/// Generate a big file with tons of snippet snippets in some of the languages listed in PROPOSED_GRAMMAR_SOURCES
/// that have
pub fn generate_large_markdown_with_codes(max_number_of_snippets_per_lang: usize) -> String {
    println!(
        "\n>> Generating for {} max number of snippets per lang\n",
        max_number_of_snippets_per_lang
    );
    let repos_folder = PathBuf::from(CODE_SNIPPETS_REPOS_DESTINATION);
    if !repos_folder.exists() {
        println!("Cloning {} under target/", CODE_SNIPPETS_REPOS);
        GitRepos::from_clone(
            CODE_SNIPPETS_REPOS,
            &repos_folder.parent().unwrap().to_path_buf(),
            Some(1),
            true,
        )
        .unwrap();
    }

    let grammars_folder = PathBuf::from(SUBFOLDER);
    if !grammars_folder.exists() {
        create_dir_all(grammars_folder).unwrap();
    }

    let mut final_output = String::from("# Sample programs in all proposed grammars languages\n");
    let mut grammars: Vec<(&&str, &&str)> =
        (*PROPOSED_GRAMMAR_SOURCES.iter().collect::<Vec<_>>()).to_vec();
    grammars.sort();
    let mut snippets_found_count = 0;
    let mut included_snippets_count = 0;
    for (lang, link) in grammars {
        if lang == &"php" || lang == &"typescript" {
            continue;
        } // it generate strange markdown outputs or doesn't support highlighting well
        let first_char = lang.chars().next().unwrap();
        let subfolder = repos_folder
            .join("archive")
            .join(first_char.to_string())
            .join(lang);

        if !subfolder.exists() {
            println!("Skipping {} because no snippets.", lang);
            continue;
        };
        let mut codes: Vec<PathBuf> = read_dir(subfolder)
            .unwrap()
            .filter(|e| e.as_ref().unwrap().path().extension() != Some(OsStr::new("md")))
            .filter(|e| e.as_ref().unwrap().path().extension() != Some(OsStr::new("yml")))
            .map(|e| e.unwrap().path())
            .collect();
        codes.sort();
        if codes.is_empty() {
            continue;
        }

        println!("Building sections for {}", lang);
        writeln!(final_output, "## Sample programs in {}", lang).unwrap();
        for code in codes.iter().take(max_number_of_snippets_per_lang) {
            writeln!(
                final_output,
                "File: `{}`\n```{}\n{}\n```",
                code.file_name().unwrap().to_str().unwrap(),
                lang,
                read_to_string(code).unwrap_or_default()
            )
            .unwrap();
            included_snippets_count += 1;
        }
        snippets_found_count += 1;
    }

    let output_md_prefix_full =
        format!("{}{}.md", OUTPUT_MD_PREFIX, max_number_of_snippets_per_lang);
    std::fs::write(&output_md_prefix_full, &final_output).unwrap();
    println!(
        "Saved file {} of size {} bytes and {} code snippets in total, with {} different languages.",
        output_md_prefix_full,
        final_output.len(),
        included_snippets_count,
        snippets_found_count
    );

    output_md_prefix_full
}
