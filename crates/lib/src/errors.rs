use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClipboardError {
    #[error("Clipboard is empty")]
    Empty,
    #[error("Invalid clipboard type: {0}")]
    InvalidType(String),
    #[error("Backend error: {0}")]
    BackendError(String),
}

#[derive(Error, Debug)]
pub enum ShrinkError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("Encoding failed: {0}")]
    EncodingFailed(String),
    #[error("Decoding failed: {0}")]
    DecodingFailed(String),
}
