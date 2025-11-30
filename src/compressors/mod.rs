pub mod bzip2;
pub mod gzip;
pub mod tar;
pub mod xz;
pub mod zip;

use std::path::Path;

use crate::core::compressor::Compressor;
use crate::core::types::CompressionFormat;

pub use bzip2::Bzip2Compressor;
pub use gzip::GzipCompressor;
pub use tar::TarCompressor;
pub use xz::XzCompressor;
pub use zip::ZipCompressor;

/// Create a compressor instance for the given format
pub fn create_compressor(format: CompressionFormat) -> Box<dyn Compressor> {
    match format {
        CompressionFormat::Gzip => Box::new(gzip::GzipCompressor::new()),
        CompressionFormat::Bzip2 => Box::new(bzip2::Bzip2Compressor::new()),
        CompressionFormat::Xz => Box::new(xz::XzCompressor::new()),
        CompressionFormat::Tar => Box::new(tar::TarCompressor::new()),
        CompressionFormat::Zip => Box::new(zip::ZipCompressor::new()),
    }
}

/// Detect compression format from file extension
pub fn detect_format(path: &Path) -> Option<CompressionFormat> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(CompressionFormat::from_extension)
}
