pub mod cli;
pub mod compressors;
pub mod core;
pub mod crypto;
pub mod operations;
pub mod utils;

// Re-export commonly used types for library users
// These are exported for external use, so allow dead_code warnings
#[allow(unused_imports)]
pub use core::{
    CollectionConfig, CollectionMode, CompressionConfig, CompressionFormat, Compressor, JcError,
    JcResult, TimestampOption,
};

#[allow(unused_imports)]
pub use operations::{
    collect_and_compress, compress_compound, compress_file, compress_files, decompress_file,
    decompress_files,
};
