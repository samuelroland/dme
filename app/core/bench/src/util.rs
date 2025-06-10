use std::process::exit;

use crate::BENCHES;

// Function to a run a function that is measured
pub fn run_fn(id: &str, args: Vec<String>) {
    match BENCHES.get(id) {
        Some(bench) => {
            println!("Running function {}", id);
            bench.1(args)
        }
        None => {
            eprintln!("Unknown function to run: {id}");
            exit(3);
        }
    }
}

// Function to run a benchmark (preparation + running hyperfine)
pub fn run_bench(id: &str) {
    match BENCHES.get(id) {
        Some(bench) => {
            println!("Running bench {}", id);
            bench.2()
        }
        None => {
            eprintln!("Unknown benchmark to run: {id}");
            exit(3);
        }
    }
}
