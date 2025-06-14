use std::{
    env::current_dir,
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::{self, sleep},
    time::{Duration, Instant},
};

use dme_core::{
    markdown_to_highlighted_html,
    preview::preview::Html,
    search::{
        disk::DiskResearcher,
        search::{Progress, ResearchResult, Researcher},
    },
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Serialize)]
struct AppInfo {
    version: String,
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
    }
}

#[tauri::command]
/// Open given Markdown file or the default one provided as argument
/// or none otherwise
fn open_markdown_file(mut path: String) -> Result<Option<Html>, String> {
    println!("{path:?}");
    if path.is_empty() {
        path = {
            let args: Vec<String> = std::env::args().collect();
            if args.len() < 2 {
                String::default()
            } else {
                args[1].clone()
            }
        }
    }

    if path.is_empty() {
        Ok(None)
    } else if PathBuf::from(&path).exists() {
        Ok(Some(markdown_to_highlighted_html(&path)?))
    } else {
        Err(format!("File {path} doesn't exist !").to_string())
    }
}

use tauri::{Builder, Manager};

struct AppData {
    disk_researcher: Mutex<DiskResearcher>,
    search_stream_receiver: Mutex<Option<Receiver<ResearchResult>>>,
}

#[tauri::command]
fn run_search(app: AppHandle, search: String) -> Result<String, String> {
    thread::spawn(move || {
        println!("Searching {search} ");
        let state = app.state::<AppData>();
        let mut ds = state.disk_researcher.lock().unwrap();
        if !ds.has_started() {
            println!("INDEXING START");
            ds.start();
        }
        drop(ds);

        let mut existing_rx = state.search_stream_receiver.lock().unwrap();

        let (tx, rx) = mpsc::channel::<ResearchResult>();
        *existing_rx = Some(rx);
        drop(existing_rx);

        // Start the thread in parallel to avoid blocking
        let app_arc = Arc::new(app);

        let app_arced = app_arc.clone();
        thread::spawn(move || {
            let state = app_arced.state::<AppData>();
            let mut progress = Progress(0);
            while !progress.is_done() {
                let guard = state.disk_researcher.lock().unwrap();
                progress = guard.progress();
                println!("Progress in indexing {}", progress.0);
                drop(guard);
                if progress.is_done() {
                    println!("INDEXING DONE");
                    break;
                } else {
                    println!("Progress in indexing {}", progress.0);
                }
                sleep(Duration::from_millis(10));
            }
        });
        let app_arced_local = app_arc.clone();
        // Span a thread to listen on rx events
        thread::spawn(move || {
            let state = app_arced_local.state::<AppData>();
            let now = Instant::now();
            loop {
                let rx_guard = state.search_stream_receiver.lock().unwrap();

                if let Some(rx) = rx_guard.as_ref() {
                    if let Ok(res) = rx.recv() {
                        println!("{:?}", res.path);
                        // print!(".");
                        app_arced_local.emit("search-match", res).unwrap();
                        sleep(Duration::from_millis(10));
                    } else {
                        println!("End of search after {:?}", now.elapsed());
                        break;
                    }
                } else {
                    break;
                }
                drop(rx_guard);
            }
        });

        let app_arced_local = app_arc.clone();

        let state = app_arc.state::<AppData>();
        let ds = state.disk_researcher.lock().unwrap();
        ds.search(&search, 10, Some(tx.clone()));
    });
    Ok("started".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            run_search,
            open_markdown_file
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
