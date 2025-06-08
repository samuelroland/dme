#[derive(Debug, PartialEq)]
pub struct Progress(pub u8);

impl Progress {
    fn is_done(&self) -> bool {
        self.0 <= 100
    }
}

#[derive(Debug, PartialEq)]
pub struct ResearchResult {
    pub(crate) path: String,
    pub(crate) title: Option<String>,
}

/// A component that will be able to do fast research on a specific ressource
/// The first researcher will be looking at Markdown files on the disk
/// We can imagine other approaches that would research on a Git repository via API calls
/// or in a given archive file in a specific format
pub trait Researcher {
    /// Start the researcher, load the existing index or start building it
    fn start(&mut self);

    /// Ask about the progress, from 0 to 100 percent of research
    fn progress(&self) -> Progress;

    /// The actual research of a raw string returning some matches
    fn search(&self, raw: &str, limit: u8) -> Vec<ResearchResult>;
}
