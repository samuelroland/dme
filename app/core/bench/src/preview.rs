use std::{fs::read_to_string, thread::sleep, time::Duration};

use dme_core::{
    markdown_to_highlighted_html,
    util::setup::{
        clone_mdn_content, generate_large_markdown_with_codes,
        install_all_grammars_in_local_target_folder,
    },
};

use crate::run_hyperfine;

// Benches and benchmarked functions
pub fn run_preview(args: Vec<String>) {
    let result = markdown_to_highlighted_html(&args[0]).unwrap();
    println!("{}", result.as_string());
}

// Benches
pub fn preview_nocode_benchmark() {
    let path = clone_mdn_content();
    // That's a file without any code snippet and of 59627 chars.
    let path = path.join("files/en-us/mdn/writing_guidelines/writing_style_guide/index.md");
    // TODO: duplicate this file a few times to make it slower to parse ?
    read_to_string(&path).unwrap();
    run_hyperfine("preview_md", vec![path.to_str().unwrap()], 100);
}

pub fn preview_code_benchmark() {
    install_all_grammars_in_local_target_folder(); // can be take 2-5 minutes the first time...

    // TODO: tweak these params to have a not too slow benchmark
    let path = generate_large_markdown_with_codes(1, 3);

    run_hyperfine("preview_codes", vec![&path], 100);
}
