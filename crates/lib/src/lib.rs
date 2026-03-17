pub mod errors;
pub mod hash;
pub mod shrink;

pub use errors::{ClipboardError, ShrinkError};
pub use hash::image_hash;
pub use shrink::{shrink, ShrinkOptions, ShrinkResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    // Compressed / lossy
    Webp,
    Jpeg,
    Avif,
    // Compressed / lossless
    Png,
    Qoi,
    Farbfeld,
    // Uncompressed
    Bmp,
    Tga,
    Tiff,
    Pnm,
    Ico,
    Gif,
    // HDR / floating-point
    Hdr,
    OpenExr,
}

pub trait ClipboardProvider {
    fn capture(&self) -> Result<(Vec<u8>, String), ClipboardError>;
    /// Distribute one or more (data, mime_type) pairs to the clipboard.
    /// Multiple pairs let receivers pick the format they support.
    fn distribute(&self, items: &[(&[u8], &str)]) -> Result<(), ClipboardError>;
}
