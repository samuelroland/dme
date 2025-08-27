use dme_core::{
    search::{disk::DiskResearcher, search::Researcher},
    util::setup::clone_mdn_content,
};

use crate::run_hyperfine;

// Benches and benchmarked functions
pub fn run_search(args: Vec<String>) {
    let mut disk_search = DiskResearcher::new(args[0].to_string());
    disk_search.start();
    let search = "abstraction";
    let results = disk_search.search(search, 20, None);
    let stats = disk_search.stats();
    dbg!(&results);
}

// Benches
pub fn general_keyword_search() {
    let repos = clone_mdn_content();
    run_hyperfine("general_keyword", vec![repos.to_str().unwrap()], 40);
}
