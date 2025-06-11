use std::{
    fs::{read_to_string, write},
    path::PathBuf,
    thread::sleep,
    time::Duration,
};

use dme_core::{
    markdown_to_highlighted_html,
    preview::tree_sitter_grammars::TreeSitterGrammarsManager,
    util::setup::{
        clone_mdn_content, generate_large_markdown_with_codes,
        install_all_grammars_in_local_target_folder,
    },
};

use crate::run_hyperfine;

// Benches and benchmarked functions
pub fn install_grammar(args: Vec<String>) {
    let mut manager = TreeSitterGrammarsManager::new().unwrap();
    manager.install(&args[0]).unwrap();
}

// Benches
pub fn grammar_install_bench() {
    // Delete possible existing Rust syntax in the global folder
    let mut manager = TreeSitterGrammarsManager::new().unwrap();
    manager.delete("rust").unwrap();
    let link = "https://github.com/tree-sitter/tree-sitter-rust";

    run_hyperfine("grammar_install", vec![link], 1);
}
