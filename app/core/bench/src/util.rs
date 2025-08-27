use std::process::exit;

use crate::BENCHES;

// Function to a run a function that is measured
pub fn run_fn(id: &str, args: Vec<String>) {
    match BENCHES.get(id) {
        Some(bench) => {
            println!("Running function {}", id);
            (bench.tested)(args)
        }
        None => {
            eprintln!("Unknown function to run: {id}");
            exit(3);
        }
    }
}
