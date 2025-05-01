// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod export;
mod preview;
mod search;

fn main() {
    dme_lib::run()
}
