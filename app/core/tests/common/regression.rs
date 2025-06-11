// Basic regression detection mecanisms, storing reference files under tests/reference
// with -ref suffix for easy
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};

/// Given a test_id used to identify tests
/// a file extension used to save existing file
/// and a new content generated during the execution of the test
/// It will make sure the new_content is equal to the last new_content sent
/// saved under reference/{test_id}-ref.{extension}
pub fn check_possible_regression(test_id: &str, extension: &str, new_content: &str) {
    let reference_folder = PathBuf::from("tests/reference");
    let filename_ref = format!("{}-ref.{}", test_id, extension);
    let filename_ref_path = reference_folder.join(&filename_ref);
    let wrong_output_path = reference_folder.join(format!("{}-wrong.{}", test_id, extension));

    // If we don't have any reference just save it
    if !filename_ref_path.exists() {
        write(filename_ref_path, new_content).unwrap();
    } else {
        // Compare with existing file, if that's store to wrong_output_path and panic
        let ref_content = &read_to_string(filename_ref_path).unwrap();
        if new_content != ref_content {
            write(&wrong_output_path, new_content).unwrap();
            // Show diff if files are small
            if new_content.len() < 1000 {
                let cmp = pretty_assertions::Comparison::new(ref_content, new_content);
                println!("{cmp}");
            }
            panic!("Previous content of {} differs with new result, saved output to {:?} for manual inspection.", filename_ref, wrong_output_path);
        } //otherwise regression test passed !
    }
}
