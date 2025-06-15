use dme_core::search::search::{Progress, ResearchResult, Researcher};
use std::sync::{mpsc, Arc};
use std::thread;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager};

use crate::AppData;

#[tauri::command]
pub fn run_search(app: AppHandle, search: String) -> Result<String, String> {
    thread::spawn(move || {
        if let Some(state) = app.try_state::<AppData>() {
            println!("Searching {search} ");
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
        };
    });
    Ok("started".to_string())
}
