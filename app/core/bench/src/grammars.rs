use dme_core::preview::tree_sitter_grammars::TreeSitterGrammarsManager;

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
