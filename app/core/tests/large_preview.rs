use common::regression::check_possible_regression;
use pretty_assertions::{assert_eq, assert_ne};
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};

use dme_core::markdown_to_highlighted_html;
use dme_core::util::setup::generate_large_markdown_with_codes;
use dme_core::*;
use regex::Replacer;
mod common;

#[test]
fn test_large_markdown_preview_with_codes_gives_same_result() {
    for i in [1, 2, 5] {
        let test_id = format!("large-preview-{}", i);
        let path = generate_large_markdown_with_codes(i, 100);

        let result = markdown_to_highlighted_html(&path).unwrap();

        check_possible_regression(&test_id, "html", result.as_string());
    }
}
