pub struct Html(String);

/// A component that will be able to preview a given document into HTML
pub trait Previewable {
    fn to_html() -> Html;
}
