/// A component that will be able to export a given document to PDF
#[allow(dead_code)]
pub trait Exportable {
    fn to_pdf(filename: String) -> Result<(), std::io::Error>;
}
