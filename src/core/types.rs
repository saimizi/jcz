use std::path::PathBuf;

/// Compression format/algorithm identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionFormat {
    Gzip,
    Bzip2,
    Xz,
    Tar,
    Zip,
}

impl CompressionFormat {
    /// Get file extension for this format
    #[allow(dead_code)]
    pub fn extension(&self) -> &'static str {
        match self {
            CompressionFormat::Gzip => "gz",
            CompressionFormat::Bzip2 => "bz2",
            CompressionFormat::Xz => "xz",
            CompressionFormat::Tar => "tar",
            CompressionFormat::Zip => "zip",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "gz" => Some(CompressionFormat::Gzip),
            "bz2" => Some(CompressionFormat::Bzip2),
            "xz" => Some(CompressionFormat::Xz),
            "tar" => Some(CompressionFormat::Tar),
            "zip" => Some(CompressionFormat::Zip),
            _ => None,
        }
    }

    /// Get algorithm name
    pub fn name(&self) -> &'static str {
        match self {
            CompressionFormat::Gzip => "gzip",
            CompressionFormat::Bzip2 => "bzip2",
            CompressionFormat::Xz => "xz",
            CompressionFormat::Tar => "tar",
            CompressionFormat::Zip => "zip",
        }
    }

    /// Create format from command name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "gzip" => Some(CompressionFormat::Gzip),
            "bzip2" => Some(CompressionFormat::Bzip2),
            "xz" => Some(CompressionFormat::Xz),
            "tar" => Some(CompressionFormat::Tar),
            "zip" => Some(CompressionFormat::Zip),
            _ => None,
        }
    }
}

/// Compound format identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundFormat {
    /// TAR + GZIP (.tar.gz)
    Tgz,

    /// TAR + BZIP2 (.tar.bz2)
    Tbz2,

    /// TAR + XZ (.tar.xz)
    Txz,
}

impl CompoundFormat {
    pub fn primary(&self) -> CompressionFormat {
        CompressionFormat::Tar
    }

    pub fn secondary(&self) -> CompressionFormat {
        match self {
            CompoundFormat::Tgz => CompressionFormat::Gzip,
            CompoundFormat::Tbz2 => CompressionFormat::Bzip2,
            CompoundFormat::Txz => CompressionFormat::Xz,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            CompoundFormat::Tgz => "tar.gz",
            CompoundFormat::Tbz2 => "tar.bz2",
            CompoundFormat::Txz => "tar.xz",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tgz" => Some(CompoundFormat::Tgz),
            "tbz2" => Some(CompoundFormat::Tbz2),
            "txz" => Some(CompoundFormat::Txz),
            _ => None,
        }
    }
}

/// Operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum OperationMode {
    Compress,
    Decompress,
}

/// Validated input file information
#[derive(Debug, Clone)]
pub struct InputFile {
    /// Original path provided by user
    #[allow(dead_code)]
    pub original_path: PathBuf,

    /// Resolved real path (after symlink resolution)
    pub real_path: PathBuf,

    /// File basename
    #[allow(dead_code)]
    pub basename: String,

    /// Whether this was a symbolic link
    #[allow(dead_code)]
    pub was_symlink: bool,
}
