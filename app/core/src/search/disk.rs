use std::collections::{HashMap, HashSet};
use std::path::{PathBuf};
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use std::ffi::OsStr;
use walkdir::WalkDir;
use crate::search::search::{Progress, ResearchResult, Researcher};

struct DiskResearcher {
    markdown_paths_set: Arc<Mutex<HashSet<String>>>,
    /// Each heading found will have an entry with a vector of files where it was found.
    title_map : Arc<Mutex<HashMap<String, Vec<String>>>>,
    base_path : PathBuf,
    nb_threads: usize,
    has_started: bool,
    threads :  Vec<thread::JoinHandle<()>>,
    progress_counter: Arc<Mutex<usize>>,
}

impl DiskResearcher {
    fn new(path: String) -> Self {
        Self {
            markdown_paths_set: Arc::new(Mutex::new(HashSet::new())),
            title_map: Arc::new(Mutex::new(HashMap::new())),
            base_path: PathBuf::from(path),
            nb_threads: num_cpus::get(),
            has_started: false,
            threads : Vec::new(),
            progress_counter: Arc::new(Mutex::new(0)),
        }
    }

    fn set_nb_thread(&mut self, nb_thread: usize) -> Result<(), String> {
        if nb_thread == 0  {
            Err("Number of thread must be greater than 0".to_string())
        } else if self.has_started {
            Err("Process has already started, cannot change thread number".to_string())
        } else{
            Ok(self.nb_threads = nb_thread)
        }

    }

    fn extract_markdown_titles(path: &str) -> Vec<String> {
        let content = fs::read_to_string(path).unwrap_or_default();

        let mut headings = Vec::new();
        let mut lines = content.lines();
        let mut is_in_code_block = false;
        while let Some(line) = lines.next() {
            if line.starts_with("~~~") || line.starts_with("```") {
                is_in_code_block = !is_in_code_block;
            }
            if is_in_code_block {
                continue;
            }
            if line.starts_with('#') {
                let line_partial = line.trim_start_matches('#');
                if line_partial.starts_with(" ") {
                    headings.push(line_partial.trim_start_matches(" ").to_string());
                }
            }
        }
        headings
    }
}

impl Researcher for DiskResearcher {
    fn start(&mut self) {
        self.has_started = true;
        //Get all paths. We have to accept the directory at first otherwise their content would be ignored
        let markdown_paths: HashSet<String> = WalkDir::new(&self.base_path.to_path_buf())
            .into_iter()
            .filter_entry(|entry|
                entry.file_type().is_dir() || entry.path().extension() == Some(OsStr::new("md"))
            )
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| {
                e.path().to_path_buf().to_str().unwrap().to_string()
            } )
            .collect();
        {
            let mut map = self.markdown_paths_set.lock().unwrap();
            *map = markdown_paths.clone();
        }

        let all_paths: Vec<_> = markdown_paths.iter().cloned().collect();
        //Calculate chunks size to divide by threads. With the + thread we ensure chunk size > 0
        if all_paths.is_empty() {
            return;
        }
        let chunk_size =  if (all_paths.len()) / self.nb_threads == 0{
            all_paths.len()
        } else {
            (all_paths.len() + self.nb_threads - 1) / self.nb_threads
        };

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
        let total = self.markdown_paths_set.lock().unwrap().len();
        if total == 0 {
            return Progress(0);
        }
        Progress((total as f32 / self.nb_threads as f32).ceil() as u8)
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

        drop(map);

        let file_list = self.markdown_paths_set.lock().unwrap();

        for file in  file_list.iter() {
            if file.contains(query.as_str()) {
                results.push(ResearchResult {
                    title: None,
                    path: file.clone().parse().unwrap(),
                });

                if results.len() >= limit.into() {
                    return results;
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

    assert!(!search.markdown_paths_set.lock().unwrap().is_empty());
    println!("{:?}", search.markdown_paths_set.lock().unwrap());
}

#[test]
fn test_that_progress_is_zero_at_start() {
    let search = DiskResearcher::new("test".parse().unwrap());
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
fn test_that_search_works_inside_files() {
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

#[test]
fn test_that_search_works_on_filename() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("depth2".to_string(), 10);
    assert_eq!(results.len(), 2);


    let results = search.search("depth3".to_string(), 10);
    assert_eq!(results.len(), 1);
}
#[test]
fn test_mixed_search() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("hello".to_string(), 10);
    assert_eq!(results.len(),2 );
}