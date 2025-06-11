use common::regression::check_possible_regression;
use dme_core::search::disk::DiskResearcher;
use dme_core::search::search::{ResearchResult, Researcher};
use pretty_assertions::{assert_eq, assert_ne};
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use dme_core::markdown_to_highlighted_html;
use dme_core::util::setup::{clone_mdn_content, generate_large_markdown_with_codes};
use dme_core::*;
use regex::Replacer;
mod common;

#[test]
fn test_large_search_on_mdn_content_can_find_multiple_match_for_generic_keyword() {
    let repos = clone_mdn_content();
    let mut disk_search = DiskResearcher::new(repos.to_str().unwrap().to_string());

    disk_search.start();
    let search = "abstraction";
    let results = disk_search.search(search, 50, None);
    let stats = disk_search.stats();
    dbg!(&stats);
    // disk_search.print_index_stats();
    assert!(stats.markdown_paths_count > 13740); // as of 2025-06-10
    assert!(stats.headings_count > 17100); // number of UNIQUE headings as of 2025-06-10
    dbg!(&results);
    let mut sorted_headings = results
        .iter()
        .filter_map(|e| e.title.clone())
        .collect::<Vec<String>>();
    sorted_headings.sort();

    // Based on command inside target/content
    // rg -o '^#{1,6} .*' --type md --no-filename | sort | sed -r "s/\#+ //g" | sort > allheadings.txt
    // grep -i "abstraction" allheadings.txt
    assert_eq!(
        sorted_headings,
        vec![
            "Advantages of Data Abstraction",
            "Control abstraction objects",
            "Control abstraction objects",
            "Larger code base and abstraction",
        ]
    );
    // Based on command inside target/content
    // fd -e md --no-ignore | grep abstraction
    assert_eq!(
        results
            .iter()
            .filter(|e| e.title.is_none())
            .map(|e| e.path.clone())
            .collect::<Vec<String>>(),
        vec![
            "target/content/files/en-us/glossary/abstraction/index.md",
            "target/content/files/en-us/web/api/idbrequest/transaction/index.md"
        ]
    );
    // assert!(results.len() >= 36, "{results:?}");
}

#[test]
fn test_large_search_on_mdn_content_can_find_a_single_heading() {
    let repos = clone_mdn_content();
    let mut disk_search = DiskResearcher::new(repos.to_str().unwrap().to_string());
    disk_search.start();
    let search = "Array constructor with a single parameter";
    let results = disk_search.search(search, 20, None);
    assert_eq!(
        results.len(),
        1,
        "Results should have only one result, contains\n{results:?}"
    );
    assert_eq!(results[0].title, Some(search.to_string()));
    assert!(results[0].path.ends_with(
        "content/files/en-us/web/javascript/reference/global_objects/array/array/index.md"
    ));
}

#[test]
fn test_large_search_on_mdn_content_can_find_a_several_headings() {
    let repos = clone_mdn_content();
    let mut disk_search = DiskResearcher::new(repos.to_str().unwrap().to_string());
    disk_search.start();
    let search = "Array constructor with"; // should match 2 headings
    let mut results = disk_search.search(search, 20, None);

    results.sort_by(|a, b| a.title.cmp(&b.title));
    assert_eq!(
        results[0].title,
        Some("Array constructor with a single parameter".to_string())
    );
    assert!(results[0].path.ends_with(
        "content/files/en-us/web/javascript/reference/global_objects/array/array/index.md"
    ));
    assert_eq!(
        results[1].title,
        Some("Array constructor with multiple parameters".to_string())
    );
    assert!(results[1].path.ends_with(
        "content/files/en-us/web/javascript/reference/global_objects/array/array/index.md"
    ));
}

#[test]
fn test_large_search_on_mdn_content_is_fuzzy_ordered_matching() {
    let repos = clone_mdn_content();
    let mut disk_search = DiskResearcher::new(repos.to_str().unwrap().to_string());
    disk_search.start();
    let patterns = vec![
        "arra c single",
        "Array constructor single",
        "constructor sgle array",
    ];
    for pattern in patterns {
        let results = disk_search.search(pattern, 20, None);
        dbg!(&results);
        assert_eq!(
            results[0].title,
            Some("Array constructor with a single parameter".to_string())
        );
        assert!(results[0].path.ends_with(
            "content/files/en-us/web/javascript/reference/global_objects/array/array/index.md"
        ));
    }
}

#[test]
fn test_large_search_on_mdn_content_is_fuzzy_on_paths() {
    let repos = clone_mdn_content();
    let mut disk_search = DiskResearcher::new(repos.to_str().unwrap().to_string());
    disk_search.start();
    let pattern = ["web js objects json index md"];
    // nothing match that in headings (checked via fzf)
    let results = disk_search.search(pattern[0], 20, None);
    dbg!(&results);
    assert!(results.len() > 4);
    dbg!(&results[0]);
    // The highest score with fuzzy is this one
    assert!(results[0].path.ends_with(
        "target/content/files/en-us/web/javascript/reference/global_objects/json/index.md"
    ));
}
