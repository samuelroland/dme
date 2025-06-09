use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Html(pub String);

impl From<String> for Html {
    fn from(value: String) -> Self {
        Html(value)
    }
}

impl Html {
    pub fn as_string(&self) -> &String {
        &self.0
    }
}

/// A component that will be able to preview a given document into HTML
pub trait Previewable<'a> {
    fn to_html(&self, source: &str) -> Html;
}
