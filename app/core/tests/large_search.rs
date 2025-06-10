use common::regression::check_possible_regression;
use dme_core::search::disk::DiskResearcher;
use dme_core::search::search::Researcher;
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
    assert!(
        results.len() >= 36,
        "Results should have only above result, contains\n{results:?}"
    );
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
    let results = disk_search.search(search, 20, None);
    assert_eq!(
        results.len(),
        2,
        "Results should have only 2 results, contains\n{results:?}"
    );
    assert_eq!(
        results[0].title,
        Some("Array constructor with a single parameter".to_string())
    );
    assert!(results[0].path.ends_with(
        "content/files/en-us/web/javascript/reference/global_objects/array/array/index.md"
    ));
    assert_eq!(
        results[0].title,
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
    let search = "Array constructor single"; // should fuzzy match the
    let results = disk_search.search(search, 20, None);
    assert_eq!(
        results.len(),
        1,
        "Results should have only one result, contains\n{results:?}"
    );
    assert_eq!(
        results[0].title,
        Some("Array constructor with a single parameter".to_string())
    );
    assert!(results[0].path.ends_with(
        "content/files/en-us/web/javascript/reference/global_objects/array/array/index.md"
    ));
}
