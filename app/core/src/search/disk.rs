use crate::search::search::{Progress, ResearchResult, Researcher};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use walkdir::WalkDir;

struct DiskResearcher {
    markdown_paths_set: Arc<Mutex<Vec<String>>>,
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
            markdown_paths_set: Arc::new(Mutex::new(Vec::new())),
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
        let markdown_paths: Vec<String> = WalkDir::new(&self.base_path.to_path_buf())
            .into_iter()
            .filter_entry(|entry|
                entry.file_type().is_dir() || entry.path().extension() == Some(OsStr::new("md"))
            )
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| {
                e.path().to_path_buf().to_str().unwrap_or_default().to_string()
            } )
            .collect();
        let all_paths: Vec<_> = markdown_paths.clone();
        {
            let mut map = self.markdown_paths_set.lock().unwrap();
            *map = markdown_paths.clone();
        }

        if all_paths.is_empty() {
            return;
        }
        let chunk_size =  if (all_paths.len()) < self.nb_threads {
            1 //This means we have more thread than the number of files
        } else {
            all_paths.len().div_ceil(self.nb_threads)
        };

        for chunk in all_paths.chunks(chunk_size) {
            let chunk = chunk.to_vec(); // copy chunk
            let title_map = Arc::clone(&self.title_map);
            let counter = Arc::clone(&self.progress_counter);

            //Create the thread to search for markdown in chunk
            let handle = thread::spawn(move || {
                //Local counter to avoid locking unlocking every loop.
                let mut  local_counter = 0;
                for path in chunk {
                    let titles = DiskResearcher::extract_markdown_titles(&path);
                    let mut map = title_map.lock().unwrap();
                    for title in titles {
                        map.entry(title).or_default().push(path.clone())
                    }
                    local_counter += 1;
                    if local_counter == 10 {
                        let mut global_counter = counter.lock().unwrap();
                        *global_counter += 10;
                        local_counter = 0;
                    }
                }
                //If final counter is not 0 then we need to add the rest
                // to the total to have the real total when finished.
                if local_counter  != 0 {
                    let mut global_counter = counter.lock().unwrap();
                    *global_counter += local_counter;
                }
            });
            self.threads.push(handle);
        }
    }
    /// Ask about the progress, from 0 to 100 percent of research
    fn progress(&self) -> Progress {
        let total = self.markdown_paths_set.lock().unwrap().len();
        if total == 0 {
            return Progress(0);
        }
        Progress(((*self.progress_counter.lock().unwrap() as f32 / total as f32) * 100f32).ceil() as u8)
    }

    /// The actual research of a raw string returning some matches
    fn search(&self, raw: &str, limit: u8) -> Vec<ResearchResult> {
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
    for path in search.markdown_paths_set.lock().unwrap().iter() {
        assert!(path.ends_with(".md"));
    }
}

#[test]
fn test_that_progress_is_zero_at_start() {
    let search = DiskResearcher::new("test".parse().unwrap());
    assert_eq!(search.progress(),Progress(0));
}

#[test]
fn test_that_progress_is_one_hundred_at_end() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    assert_eq!(search.progress(),Progress(100));
}

#[test]
fn test_that_search_works_inside_files() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("Hello world!", 10);
    assert_eq!(results.len(), 0);

    let results2 = search.search("Introduction", 10);
    assert_eq!(results2.len(), 2);
    assert!(results2.contains(&ResearchResult { path: "test/depth2/test.md".to_string(), title: Some("Introduction".to_string())}));
    assert!(results2.contains(&ResearchResult{path : "test/depth1/test.md".to_string(), title: Some("Introduction".to_string())}));
    let results2 = search.search("intro", 10);

    assert!(results2.contains(&ResearchResult { path: "test/depth2/test.md".to_string(), title: Some("Introduction".to_string())}));
    assert!(results2.contains(&ResearchResult{path : "test/depth1/test.md".to_string(), title: Some("Introduction".to_string())}));
    assert!(results2.contains(&ResearchResult { path: "test/depth1/test.md".to_string(), title: Some("Intro".to_string())}));
    assert!(results2.contains(&ResearchResult{path : "test/depth1/test.md".to_string(), title: Some("I swear introspection".to_string())}));
    assert_eq!(results2.len(), 4);
}

#[test]
fn test_that_search_works_on_filename() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("depth2", 10);
    assert_eq!(results.len(), 2);
    assert!(results.contains(&ResearchResult { path: "test/depth2/test.md".to_string(), title: None}));
    assert!(results.contains(&ResearchResult{path : "test/depth2/depth3/test3.md".to_string(), title: None}));


    let results = search.search("depth3", 10);
    assert_eq!(results.len(), 1);
    assert!(results.contains(&ResearchResult{path : "test/depth2/depth3/test3.md".to_string(), title: None}));

}
#[test]
fn test_mixed_search() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("hello", 10);
    assert_eq!(results.len(),2 );
    assert!(results.contains(&ResearchResult{path : "test/depth1/hello.md".to_string(), title: None}));
    assert!(results.contains(&ResearchResult{path : "test/depth1/test4.md".to_string(), title: Some("Hello".to_string())}));

}

#[test]
fn test_that_number_of_thread_is_not_higher_than_necessary() {
    let mut search = DiskResearcher::new("test".to_string());
    search.set_nb_thread(100).unwrap();
    search.start();
    thread::sleep(std::time::Duration::from_secs(2));
    assert_eq!(search.threads.len(),6);
}

#[test]
fn test_that_number_of_thread_is_not_higher_than_set() {
    let mut search = DiskResearcher::new("test".to_string());
    search.set_nb_thread(2).unwrap();
    search.start();
    thread::sleep(std::time::Duration::from_secs(2));
    assert_eq!(search.threads.len(),2);
}
#[test]
fn test_that_empty_directory_cause_no_issue() {
    let mut search = DiskResearcher::new("test/depth2/depth3/depth4/".to_string());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("hello", 10);
    assert_eq!(results.len(),0 );
}

#[test]
fn test_that_limit_works() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("hello", 1);
    assert_eq!(results.len(),1);
}