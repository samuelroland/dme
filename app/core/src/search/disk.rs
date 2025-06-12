use crate::search::search::{Progress, ResearchResult, Researcher};
use crate::util::setup::clone_mdn_content;
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::collections::{BinaryHeap, HashMap};
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use walkdir::WalkDir;

use super::search::IndexStat;

impl PartialOrd for ResearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ResearchResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

/// Storing the results incrementally found in the search index
#[derive(Clone)]
struct OrderedResults {
    results: BinaryHeap<ResearchResult>,
    tx: Option<Sender<ResearchResult>>,
}

impl OrderedResults {
    pub fn new(tx: Option<Sender<ResearchResult>>) -> Self {
        Self {
            results: BinaryHeap::default(),
            tx,
        }
    }

    pub fn push(&mut self, result: ResearchResult) {
        self.results.push(result.clone());
        if let Some(tx) = &self.tx {
            let err = tx.send(result.clone());
            if err.is_err() {
                //This means the channel is closed, or we cannot write on it
                //Either stop sending message to the channel
                self.tx = None;
            }
        }
    }

    /// First `limit` results from internal partial ordered list of results
    /// Returns less results in case there are not relevant enough
    pub fn results(&self, max_limit: usize) -> Vec<ResearchResult> {
        let mut heap = self.results.clone();
        let results: Vec<ResearchResult> = (0..max_limit).filter_map(|_| heap.pop()).collect();
        let max = results.iter().max_by(|a, b| a.priority.cmp(&b.priority));

        // Filter results to only take the upper quarter of best matches. This quarter is defined
        // between max_priority - max_priority/4, and max_priority.
        // Considering list of matches priorities: 126 234 523 663, we only want the last 2 because
        // the first 2 values are too far away from the maximum and probably not relevant results
        if let Some(res) = max {
            let m = res.priority;
            let imposed_min = m - (((m as f32) / 4.) as u32);
            return results
                .into_iter()
                .filter(|e| e.priority >= imposed_min)
                .collect();
        }
        results
    }

    pub fn len(&self) -> usize {
        self.results.len()
    }
}

#[derive(Debug)]
pub struct DiskResearcher {
    /// The list of all paths to Markdown files found
    /// Use a RwLock to make searches more optimized during index construction
    markdown_paths_vec: Arc<RwLock<Vec<String>>>,
    /// Each heading found will have an entry with a vector of files where it was found.
    /// Use a RwLock to make searches more optimized during index construction
    title_map: Arc<RwLock<HashMap<String, Vec<String>>>>,
    base_path: PathBuf,
    max_nb_threads: usize,
    has_started: bool,
    /// The progress counter counting the number of markdown paths where the titles
    /// have been extracted and saved
    /// Use a Mutex because writes will probably be more frequent than reads
    progress_counter: Arc<Mutex<usize>>,
}

impl DiskResearcher {
    pub fn new(path: String) -> Self {
        Self {
            markdown_paths_vec: Arc::new(RwLock::new(Vec::new())),
            title_map: Arc::new(RwLock::new(HashMap::new())),
            base_path: PathBuf::from(path),
            max_nb_threads: num_cpus::get(),
            has_started: false,
            progress_counter: Arc::new(Mutex::new(0)),
        }
    }

    pub fn set_max_nb_threads(&mut self, nb_thread: usize) -> Result<(), String> {
        if nb_thread == 0 {
            Err("Number of thread must be greater than 0".to_string())
        } else if self.has_started {
            Err("Process has already started, cannot change thread number".to_string())
        } else {
            self.max_nb_threads = nb_thread;
            Ok(())
        }
    }

    pub fn extract_markdown_titles(content: &str) -> Vec<String> {
        if content.is_empty() {
            return vec![];
        }
        let mut headings = Vec::new();
        let lines = content.lines();
        let mut is_in_code_block = false;
        for line in lines {
            if line.starts_with("~~~") || line.starts_with("```") {
                is_in_code_block = !is_in_code_block;
            }
            if is_in_code_block {
                continue;
            }
            if line.starts_with('#') {
                let line_partial = line.trim_start_matches('#');
                if line_partial.starts_with(" ") {
                    headings.push(line_partial.trim().to_string());
                }
            }
        }
        headings
    }
}

impl Researcher for DiskResearcher {
    fn start(&mut self) {
        self.has_started = true;

        // Get all paths by searching for Marddown files on disk
        // We have to accept the directory at first otherwise their content would be ignored
        let markdown_paths: Vec<String> = WalkDir::new(&self.base_path)
            .into_iter()
            .filter_entry(|entry| {
                entry.file_type().is_dir() || entry.path().extension() == Some(OsStr::new("md"))
            })
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| {
                e.path()
                    .to_path_buf()
                    .to_str()
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();
        // Save the result into the markdown_paths_set
        {
            let mut map = self.markdown_paths_vec.write().unwrap();
            *map = markdown_paths.clone();
        }

        if markdown_paths.is_empty() {
            return;
        }
        let chunk_size = if (markdown_paths.len()) < self.max_nb_threads {
            1 //This means we have more thread than the number of files
        } else {
            markdown_paths.len().div_ceil(self.max_nb_threads)
        };

        // We iterate on chunks and do copy of it, because we need to have a local version on each
        // thread to iterate without needing to lock the RwLock to read each entry
        // and potentially be blocked by writers threads during indexing
        for chunk in markdown_paths.chunks(chunk_size) {
            let local_chunk = chunk.to_vec(); // copy chunk
            let title_map = Arc::clone(&self.title_map);
            let counter = Arc::clone(&self.progress_counter);

            // Create the thread to search for markdown in chunk
            thread::spawn(move || {
                // Local counter to avoid locking unlocking every loop.
                let mut local_counter = 0;
                for path in local_chunk {
                    let content = read_to_string(&path).unwrap_or_default();
                    // We found some titles, let's insert them or add their paths to existing entries
                    let titles = DiskResearcher::extract_markdown_titles(&content);
                    {
                        let mut map = title_map.write().unwrap();
                        for title in titles {
                            map.entry(title).or_default().push(path.clone())
                        }
                    }
                    local_counter += 1;
                    // We update the shared counter not at each iteration
                    if local_counter == 10 {
                        {
                            let mut global_counter = counter.lock().unwrap();
                            *global_counter += 10;
                        }
                        local_counter = 0;
                    }
                }
                // If final counter is not 0 then we need to add the rest
                // to the total to have the real total when finished.
                if local_counter != 0 {
                    let mut global_counter = counter.lock().unwrap();
                    *global_counter += local_counter;
                }
            });
        }
    }

    /// Ask about the progress, from 0 to 100 percent of research
    fn progress(&self) -> Progress {
        let total = self.markdown_paths_vec.read().unwrap().len();
        if total == 0 {
            return Progress(0);
        }
        Progress(
            ((*self.progress_counter.lock().unwrap() as f32 / total as f32) * 100f32).ceil() as u8,
        )
    }

    /// The actual research of a raw string returning some matches
    fn search(
        &self,
        raw: &str,
        limit: u8,
        sender: Option<Sender<ResearchResult>>,
    ) -> Vec<ResearchResult> {
        let map = self.title_map.read().unwrap().clone();

        let results: Arc<Mutex<OrderedResults>> = Arc::new(Mutex::new(OrderedResults::new(sender)));
        let headings: Vec<_> = map.into_iter().collect();

        let mut threads = Vec::new();
        let chunk_size = if headings.len() < self.max_nb_threads {
            1
        } else {
            headings.len().div_ceil(self.max_nb_threads)
        };

        let query = raw.to_lowercase();
        // Each thread will inspect part of the titles index to search
        // for matches of the given query. Again we use a local copy.
        for tuples_chunk in headings.chunks(chunk_size) {
            let tuples = tuples_chunk.to_vec(); // copy chunk
            let results = Arc::clone(&results);

            let local_query = query.clone();
            let handle = thread::spawn(move || {
                let mut matcher = Matcher::new(Config::DEFAULT);
                // Search in parallel into the headings
                let pattern = Pattern::new(
                    &local_query,
                    CaseMatching::Ignore,
                    Normalization::Smart,
                    AtomKind::Fuzzy,
                );
                for (title, paths) in tuples {
                    let mut chars: Vec<char> = Vec::new();
                    let ascii_title = Utf32Str::new(&title, &mut chars);
                    let score = pattern.score(ascii_title, &mut matcher).unwrap_or(0);
                    if score > 10 {
                        for path in paths.iter() {
                            let mut results = results.lock().unwrap();
                            results.push(ResearchResult {
                                title: Some(title.clone()),
                                path: path.to_string(),
                                priority: score,
                            });
                        }
                    }
                }
            });

            threads.push(handle);
        }

        // Search in parallel in the path as well and attibute higher priority
        let file_list = self.markdown_paths_vec.read().unwrap().clone();
        let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
        // Search in parallel into the headings
        let pattern = Pattern::new(
            &query,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        for file in file_list.iter() {
            let mut chars: Vec<char> = Vec::new();
            let ascii_title = Utf32Str::new(file, &mut chars);
            let score = pattern.score(ascii_title, &mut matcher).unwrap_or(0);
            if score > 10 {
                results.lock().unwrap().push(ResearchResult {
                    title: None,
                    path: file.clone().parse().unwrap(),
                    priority: (score as f32 * 1.3) as u32,
                });
            }
        }
        for thread in threads {
            thread.join().unwrap();
        }
        let final_results = results.lock().unwrap().clone();
        final_results.results(limit as usize)
    }

    fn stats(&self) -> IndexStat {
        IndexStat {
            headings_count: self.title_map.read().unwrap().len(),
            markdown_paths_count: self.markdown_paths_vec.read().unwrap().len(),
        }
    }
}

#[test]
fn test_that_file_are_found() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    //Wait for completion
    thread::sleep(std::time::Duration::from_secs(1));

    assert!(!search.markdown_paths_vec.read().unwrap().is_empty());
    for path in search.markdown_paths_vec.read().unwrap().iter() {
        assert!(path.ends_with(".md"));
    }
}

#[test]
fn test_that_progress_is_zero_at_start() {
    let search = DiskResearcher::new("test".parse().unwrap());
    assert_eq!(search.progress(), Progress(0));
}

#[test]
fn test_that_progress_is_one_hundred_at_end() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    assert_eq!(search.progress(), Progress(100));
}

#[test]
fn test_heading_extractions() {
    let content = "
# heyo  
some content ** ## yoo 
## oups \t
```bash
# super comment
echo saasdf
```

### 4. Open a pull request with your example

# aa";

    assert_eq!(
        DiskResearcher::extract_markdown_titles(content),
        vec![
            "heyo",
            "oups",
            "4. Open a pull request with your example",
            "aa"
        ]
    );
}

#[test]
fn test_heading_extractions_advanced() {
    let repos = clone_mdn_content();
    let path = repos.join("files/en-us/web/css/layout_cookbook/contribute_a_recipe/index.md");
    let expected = vec![
        "What makes a good recipe?",
        "Steps to publish a recipe",
        "1. Build a pattern",
        "2. Create a live example",
        "Useful tips",
        "3. Create a downloadable version",
        "4. Open a pull request with your example",
        "5. Create your page",
        "See also",
    ];
    assert_eq!(
        DiskResearcher::extract_markdown_titles(&read_to_string(path).unwrap()),
        expected
    );
}

#[test]
fn test_that_search_works_inside_files() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("Hello world!", 10, None);
    assert_eq!(results.len(), 0);

    let results2 = search.search("Introduction", 10, None);
    assert_eq!(results2.len(), 2);

    assert!(results2.contains(&ResearchResult {
        path: "test/depth2/test.md".to_string(),
        title: Some("Introduction".to_string()),
        priority: 322,
    }));
    assert!(results2.contains(&ResearchResult {
        path: "test/depth1/test.md".to_string(),
        title: Some("Introduction".to_string()),
        priority: 322,
    }));
    let results2 = search.search("intro", 10, None);

    assert!(results2.contains(&ResearchResult {
        path: "test/depth2/test.md".to_string(),
        title: Some("Introduction".to_string()),
        priority: 140,
    }));
    assert!(results2.contains(&ResearchResult {
        path: "test/depth1/test.md".to_string(),
        title: Some("Introduction".to_string()),
        priority: 140,
    }));
    assert!(results2.contains(&ResearchResult {
        path: "test/depth1/test.md".to_string(),
        title: Some("Intro".to_string()),
        priority: 140,
    }));
    assert!(results2.contains(&ResearchResult {
        path: "test/depth1/test.md".to_string(),
        title: Some("I swear introspection".to_string()),
        priority: 140,
    }));
    assert_eq!(results2.len(), 4);
}

#[test]
fn test_that_search_works_on_filename() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("depth2", 10, None);
    assert_eq!(results.len(), 2);

    assert!(results.contains(&ResearchResult {
        path: "test/depth2/test.md".to_string(),
        title: None,
        priority: 206,
    }));
    assert!(results.contains(&ResearchResult {
        path: "test/depth2/depth3/test3.md".to_string(),
        title: None,
        priority: 206,
    }));

    let results = search.search("depth3", 10, None);

    assert_eq!(results.len(), 1);
    assert!(results.contains(&ResearchResult {
        path: "test/depth2/depth3/test3.md".to_string(),
        title: None,
        priority: 206,
    }));
}
#[test]
fn test_mixed_search() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("hello", 10, None);
    assert_eq!(results.len(), 2);

    assert!(results.contains(&ResearchResult {
        path: "test/depth1/hello.md".to_string(),
        title: None,
        priority: 174,
    }));
    assert!(results.contains(&ResearchResult {
        path: "test/depth1/test4.md".to_string(),
        title: Some("Hello".to_string()),
        priority: 140,
    }));
}

#[test]
fn test_that_empty_directory_cause_no_issue() {
    let mut search = DiskResearcher::new("test/depth2/depth3/depth4/".to_string());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("hello", 10, None);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_that_limit_works() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("hello", 1, None);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_priority_is_respected() {
    let mut search = DiskResearcher::new("test".parse().unwrap());
    search.start();
    thread::sleep(std::time::Duration::from_secs(1));
    let results = search.search("t", 100, None);
    assert!(results.is_sorted_by(|a, b| a.priority >= b.priority));
}
