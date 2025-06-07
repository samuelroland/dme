use std::collections::{HashMap, HashSet};
use std::path::{PathBuf};
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use walkdir::WalkDir;
use regex::Regex;
use crate::search::search::{Progress, ResearchResult, Researcher};

struct DiskResearcher {
    markdown_map : Arc<Mutex<HashSet<String>>>,
    title_map : Arc<Mutex<HashMap<String, Vec<String>>>>,
    base_path : PathBuf,
    nb_thread: usize,
    has_started: bool,
    threads :  Vec<thread::JoinHandle<()>>,
    progress_counter: Arc<Mutex<usize>>,
}

impl DiskResearcher {
    fn new(path: String) -> Self {
        Self {
            markdown_map: Arc::new(Mutex::new(HashSet::new())),
            title_map: Arc::new(Mutex::new(HashMap::new())),
            base_path: PathBuf::from(path),
            nb_thread : num_cpus::get(),
            has_started: false,
            threads : Vec::new(),
            progress_counter: Arc::new(Mutex::new(0)),
        }
    }

    fn set_nb_thread(&mut self, nb_thread: usize) {
        assert!(nb_thread > 0);
        assert!(!self.has_started);
        self.nb_thread = nb_thread;
    }

    fn extract_markdown_titles(path: &str) -> Vec<String> {
        let content = fs::read_to_string(path).unwrap_or_default();

        let heading_regex = Regex::new(r"^#+\s+(.+$)").unwrap();

        content
            .lines()
            .filter(|line| heading_regex.is_match(line))
            .map(|line|
                     heading_regex
                         .captures(line)
                         .and_then(|caps| caps.get(1))
                         .map(|m|m.as_str().trim().to_string())
                         .unwrap_or_default()
            )
            .collect()
    }
}

impl Researcher for DiskResearcher {
    fn start(&mut self) {
        let base_path = self.base_path.to_path_buf();
        self.has_started = true;
        //Get all paths
        let markdown_paths = WalkDir::new(&base_path)
            .into_iter()
            .filter_entry(|entry| {
                entry.file_type().is_dir() || entry.file_name().to_str().unwrap().ends_with(".md")
            })
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf().to_str().unwrap().to_owned())
            .collect();
        {
            let mut map = self.markdown_map.lock().unwrap();
            *map = markdown_paths;
        }
        let map_guard = self.markdown_map.lock().unwrap();
        let all_paths: Vec<_> = map_guard.iter().cloned().collect();
        drop(map_guard);

        //Calculate chunks size to divide by threads. With the + thread we ensure chunk size > 0
        let chunk_size =  (all_paths.len() + self.nb_thread - 1) / self.nb_thread;
        for chunk in all_paths.chunks(chunk_size) {
            let chunk = chunk.to_vec(); // copy chunk
            let title_map = Arc::clone(&self.title_map);
            let counter = Arc::clone(&self.progress_counter);

            //Create the thread to search for markdown in chunk
            let handle = thread::spawn(move || {
                for path in chunk {
                    let titles = DiskResearcher::extract_markdown_titles(&path);
                    let mut map = title_map.lock().unwrap();
                    for title in titles {
                        map.entry(title).or_default().push(path.clone().to_string())
                    }
                    let mut count = counter.lock().unwrap();
                    *count += 1;
                }
            });
            self.threads.push(handle);
        }
        //Once all thread are started, spawn one to say when finding is finished

    }
    /// Ask about the progress, from 0 to 100 percent of research
    fn progress(&self) -> Progress {
        let total = self.markdown_map.lock().unwrap().len();
        if total == 0 {
            return Progress(0);
        }
        Progress((total as f32 / self.nb_thread as f32).ceil() as u8)
    }

    /// The actual research of a raw string returning some matches
    fn search(&self, raw: String, limit: u8) -> Vec<ResearchResult> {
        let query = raw.to_lowercase();
        let map = self.title_map.lock().unwrap();

        let mut results = Vec::new();

        for (title, paths) in map.iter() {
            if title.to_lowercase().contains(&query) {
                for path in paths {
                    results.push(ResearchResult {
                        title: Some(title.clone()),
                        path: path.clone().parse().unwrap(),
                    });

                    if results.len() >= limit.into() {
                        return results;
                    }
                }
            }
        }

        results
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

#[test]
fn test_that_progress_is_zero_at_start() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    assert_eq!(search.progress(),Progress(0));
}

#[test]
fn test_that_progress_is_one_at_end() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    assert_eq!(search.progress(),Progress(1));
}

#[test]
fn test_that_search_works() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("Hello world!".to_string(), 10);
    assert_eq!(results.len(), 0);

    let resutts2 = search.search("Introduction".to_string(), 10);
    assert_eq!(resutts2.len(), 2);

    let resutts2 = search.search("intro".to_string(), 10);
    assert_eq!(resutts2.len(), 4);

}