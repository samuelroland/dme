use std::{
    fs::{read_to_string, write},
    path::PathBuf,
};

use dme_core::{
    markdown_file_to_highlighted_html,
    util::setup::{
        clone_mdn_content, generate_large_markdown_with_codes,
        install_all_grammars_in_local_target_folder,
    },
};

use crate::run_hyperfine;

// Benches and benchmarked functions
pub fn run_preview(args: Vec<String>) {
    let result = markdown_file_to_highlighted_html(&args[0]).unwrap();
    println!("{}", result.to_safe_html_string());
}

// Benches
pub fn preview_nocode_benchmark() {
    let path = clone_mdn_content();
    // That's a file without any code snippet and of 59627 chars.
    let path = path.join("files/en-us/mdn/writing_guidelines/writing_style_guide/index.md");

    let mut content = read_to_string(&path).unwrap();
    let base = content.clone();
    for _ in 1..30 {
        content += &base;
    }
    let destination = PathBuf::from("target/big_markdown.md");
    write(&destination, content).unwrap();
    run_hyperfine("preview_md", vec![destination.to_str().unwrap()], 20);
}

pub fn preview_code_benchmark() {
    install_all_grammars_in_local_target_folder(); // can be take 2-5 minutes the first time...
    let path = generate_large_markdown_with_codes(30, 15);
    run_hyperfine("preview_code", vec![&path], 20);
}
