mod commands;

use crate::commands::grammars::get_grammars_list;
use crate::commands::search::run_search;
use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::{self, sleep},
    time::{Duration, Instant},
};

use commands::{
    grammars::{grammars_folder, install_grammar, remove_grammar},
    home::get_app_info,
    preview::open_markdown_file,
};
use dme_core::{
    markdown_to_highlighted_html,
    preview::{
        preview::Html, proposed_grammars::PROPOSED_GRAMMAR_SOURCES,
        tree_sitter_grammars::TreeSitterGrammarsManager,
    },
    search::{
        disk::DiskResearcher,
        search::{Progress, ResearchResult, Researcher},
    },
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri::{Builder, Manager};

struct AppData {
    disk_researcher: Mutex<DiskResearcher>,
    search_stream_receiver: Mutex<Option<Receiver<ResearchResult>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            run_search,
            open_markdown_file,
            get_grammars_list,
            install_grammar,
            remove_grammar,
            grammars_folder
        ])
        .setup(|app| {
            let home_dir = etcetera::home_dir()
                .expect("Couldn't get HOME directory")
                .to_str()
                .unwrap_or_default()
                .to_string();
            let disk_researcher = DiskResearcher::new(home_dir);
            app.manage(AppData {
                disk_researcher: Mutex::new(disk_researcher),
                search_stream_receiver: Mutex::new(None),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
