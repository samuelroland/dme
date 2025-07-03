use common::regression::check_possible_regression;
use dme_core::util::setup::{
    generate_large_markdown_with_codes, install_all_grammars_in_local_target_folder,
};
use dme_core::*;
mod common;

#[test]
fn test_large_markdown_preview_with_codes_gives_same_result() {
    install_all_grammars_in_local_target_folder();
    for i in [1, 2, 5] {
        let test_id = format!("large-preview-{i}");
        let path = generate_large_markdown_with_codes(i, 100);

        let result = markdown_file_to_highlighted_html(&path).unwrap();

        check_possible_regression(&test_id, "html", result.as_string());
    }
}
