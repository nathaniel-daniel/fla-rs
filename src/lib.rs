pub mod fla;
pub mod types;

pub use crate::fla::Fla;

/// Result type
pub type FlaResult<T> = Result<T, FlaError>;

/// Error Type
#[derive(Debug, thiserror::Error)]
pub enum FlaError {
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),

    #[error(transparent)]
    Xml(#[from] quick_xml::DeError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
