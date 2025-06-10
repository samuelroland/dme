// This is our benchmark, driven by a hyperfine wrapper
use std::{collections::HashMap, process::Command};
mod preview;
mod util;

use colored::Colorize;
use dme_core::{
    markdown_to_highlighted_html,
    util::setup::{
        clone_mdn_content, generate_large_markdown_with_codes,
        install_all_grammars_in_local_target_folder,
    },
};
use once_cell::sync::Lazy;
use preview::{preview_code_benchmark, preview_nocode_benchmark, run_preview};
use util::run_fn;

fn run_hyperfine(fn_id: &str, program_args: Vec<&str>, runs: usize) {
    let args: Vec<String> = std::env::args().collect();
    let handle = Command::new("hyperfine")
        .args(vec![
            "-N",
            "-r",
            &runs.to_string(),
            // Benchmark the same binary as the current one but with other args
            &format!("{} fn {} {}", args[0], fn_id, program_args.join(" ")),
        ])
        .spawn();
    handle.unwrap().wait().unwrap();
}

pub struct Bench {
    pub desc: &'static str,
    pub tested: fn(Vec<String>),
    pub bench: fn(),
}

pub static BENCHES: Lazy<HashMap<&'static str, Bench>> = Lazy::new(|| {
    HashMap::from([
        (
            "preview_md",
            Bench {
                desc: "Large Markdown file without code snippets",
                tested: run_preview as fn(Vec<String>),
                bench: preview_nocode_benchmark as fn(),
            },
        ),
        (
            "preview_code",
            Bench {
                desc: "Different code snippets numbers in various languages",
                tested: run_preview as fn(Vec<String>),
                bench: preview_code_benchmark as fn(),
            },
        ),
    ])
});

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Run all benchmarks
    if args.len() == 1 {
        println!("Listing available benchmarks");
        for (id, v) in BENCHES.iter() {
            println!("- {} : {}", id, v.desc);
        }

        println!("To execute a benchmark run: cargo run --release -- bench <id>")
    } else {
        // Run a given function and forward args after the function id
        if args.len() >= 3 && args[1] == "fn" {
            let id = args[2].clone();
            run_fn(&id, args.into_iter().skip(3).collect());
            return;
        }
        // Run all benchmarks
        if args.len() >= 2 && args[1] == "bench" {
            if args.len() == 2 {
                println!("Running all benches");
                let mut keys: Vec<&&str> = BENCHES.keys().collect();
                keys.sort();
                for key in keys {
                    println!("Running bench {}", key.green());
                    (BENCHES
                        .get(key)
                        .expect(&format!("No benchmark with name {}", key))
                        .bench)();
                }
            } else {
                // Run a specific bench
                println!("Running bench {}", args[2].green());
                (BENCHES
                    .get(args[2].as_str())
                    .expect(&format!("No benchmark with name {}", &args[2]))
                    .bench)();
            }
        }
    }
}
