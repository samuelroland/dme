use std::{
    fs::{read_to_string, write},
    path::PathBuf,
    thread::sleep,
    time::Duration,
};

use dme_core::{
    markdown_to_highlighted_html,
    search::{disk::DiskResearcher, search::Researcher},
    util::setup::{
        clone_mdn_content, generate_large_markdown_with_codes,
        install_all_grammars_in_local_target_folder,
    },
};

use crate::run_hyperfine;

// Benches and benchmarked functions
pub fn run_search(args: Vec<String>) {
    let mut disk_search = DiskResearcher::new(args[0].to_string());
    disk_search.start();
    let search = "abstraction";
    // let results = disk_search.search(search, 20, None);
    // let stats = disk_search.stats();
    // dbg!(&results);
}

// Benches
pub fn general_keyword_search() {
    let repos = clone_mdn_content();
    run_hyperfine("general_keyword", vec![repos.to_str().unwrap()], 40);
}
