use std::collections::HashMap;
use std::path::{PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;
use crate::search::search::{Progress, ResearchResult, Researcher};

struct DiskResearcher {
    markdown_map : Arc<Mutex<HashMap<String,PathBuf>>>,
    base_path : PathBuf,
    nb_thread: u16,
    has_started: bool,
    threads :  Vec<thread::JoinHandle<()>>,
}

impl DiskResearcher {
    fn new(path: String) -> Self {
        Self {
            markdown_map: Arc::new(Mutex::new(HashMap::new())),
            base_path: PathBuf::from(path),
            nb_thread : 4,
            has_started: false,
            threads : Vec::new(),
        }
    }

    fn set_nb_thread(&mut self, nb_thread: u16) {
        assert!(nb_thread > 0);
        self.nb_thread = nb_thread;
    }
}

impl Researcher for DiskResearcher {
    fn start(&mut self) {
        let base_path = self.base_path.to_path_buf();
        //Get all paths
        let all_paths: Vec<PathBuf> = WalkDir::new(&base_path)
            .into_iter()
            .filter_map(Result::ok)
            .map(|path| path.path().to_path_buf())
            .collect();
        //Calculate chunks size to divide by threads
        let chunk_size = all_paths.len() / self.nb_thread as usize;
        for chunk in all_paths.chunks(chunk_size) {
            let chunk = chunk.to_vec(); // copy chunk
            let map_clone = Arc::clone(&self.markdown_map);
            let base_clone = base_path.clone();

            //Create the thread to search for markdown in chunk
            let handle = thread::spawn(move || {
                for path in chunk {
                    if path.extension().map_or(false, |ext| ext == "md") {
                        let relative = path.strip_prefix(&base_clone)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string();

                        map_clone.lock().unwrap().insert(relative, path);
                    }
                }
            });
            self.threads.push(handle);
        }
    }
    /// Ask about the progress, from 0 to 100 percent of research
    fn progress() -> Progress {
        todo!()
    }

    /// The actual research of a raw string returning some matches
    fn search(raw: String) -> Vec<ResearchResult> {
        todo!()
    }
}

#[test]
fn test_that_file_are_found() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    //Wait for completion
    thread::sleep(std::time::Duration::from_secs(1));

    assert!(!search.markdown_map.lock().unwrap().is_empty());
    println!("{:?}", search.markdown_map.lock().unwrap());
}

