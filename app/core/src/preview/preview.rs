pub struct Html(pub String);

impl From<String> for Html {
    fn from(value: String) -> Self {
        Html(value)
    }
}

/// A component that will be able to preview a given document into HTML
pub trait Previewable {
    fn to_html() -> Html;
}
