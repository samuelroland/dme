/// A component that will be able to export a given document to PDF
pub trait Exportable {
    fn to_pdf(filename: String) -> Result<(), std::io::Error>;
}
