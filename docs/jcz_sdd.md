# Software Design Document (SDD)
## JCZ - Just Compress Zip Utility (Rust Implementation)

**Version:** 1.1
**Date:** 2025-11-30
**Document Status:** Final
**Implementation Language:** Rust

---

## 1. Introduction

### 1.1 Purpose
This Software Design Document describes the architecture, design, and implementation strategy for the JCZ (Just Compress Zip) utility in Rust. It provides detailed technical specifications for developers implementing the system.

### 1.2 Scope
This document covers:
- System architecture and module design
- Data structures and interfaces
- Algorithm implementations
- Rust-specific design patterns and idioms
- Error handling strategies
- Concurrency model
- Testing approach

### 1.3 References
- JCZ Software Requirements Specification (jcz_srs.md)
- Rust Programming Language Documentation
- The Rust Book (https://doc.rust-lang.org/book/)
- Rust API Guidelines (https://rust-lang.github.io/api-guidelines/)

### 1.4 Design Principles

#### Rust-Specific Principles
1. **Memory Safety**: Leverage Rust's ownership system for safe memory management
2. **Zero-Cost Abstractions**: Use traits and generics without runtime overhead
3. **Fearless Concurrency**: Use Rust's type system to prevent data races
4. **Error Handling**: Use `Result<T, E>` for recoverable errors
5. **Type Safety**: Leverage strong typing to prevent logic errors at compile time

#### Software Engineering Principles
1. **Modularity**: Clear separation of concerns
2. **Single Responsibility**: Each module has one well-defined purpose
3. **Open/Closed Principle**: Open for extension, closed for modification
4. **DRY (Don't Repeat Yourself)**: Shared functionality in common modules
5. **KISS (Keep It Simple)**: Avoid unnecessary complexity

---

## 2. System Architecture

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│  (Argument Parsing, User Interface, Command Dispatch)       │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│     (Business Logic, Workflow Orchestration)                 │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                  Compression Abstraction                     │
│  (Trait Definitions, Common Interfaces, Type Dispatch)       │
└──────────────────────┬──────────────────────────────────────┘
                       │
         ┌─────────────┼─────────────┬────────────┬────────────┐
         ▼             ▼             ▼            ▼            ▼
    ┌────────┐   ┌────────┐   ┌────────┐   ┌────────┐   ┌────────┐
    │  GZIP  │   │ BZIP2  │   │   XZ   │   │  ZIP   │   │  TAR   │
    │ Module │   │ Module │   │ Module │   │ Module │   │ Module │
    └────────┘   └────────┘   └────────┘   └────────┘   └────────┘
         │             │             │            │            │
         └─────────────┴─────────────┴────────────┴────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                   Utility/Support Layer                      │
│  (Logging, File I/O, Process Execution, Validation)         │
└─────────────────────────────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    External Dependencies                     │
│        (std::process, std::fs, rayon, clap, etc.)           │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Module Structure

```
jcz/
├── src/
│   ├── main.rs                 # Entry point, CLI setup
│   ├── lib.rs                  # Library root, public API
│   ├── cli/
│   │   ├── mod.rs              # CLI module root
│   │   ├── args.rs             # Argument parsing (clap)
│   │   └── commands.rs         # Command handlers
│   ├── core/
│   │   ├── mod.rs              # Core module root
│   │   ├── config.rs           # Configuration structures
│   │   ├── compressor.rs       # Compressor trait definition
│   │   ├── error.rs            # Error types and handling
│   │   └── types.rs            # Common type definitions
│   ├── compressors/
│   │   ├── mod.rs              # Compressor module root
│   │   ├── gzip.rs             # GZIP implementation
│   │   ├── bzip2.rs            # BZIP2 implementation
│   │   ├── xz.rs               # XZ implementation
│   │   ├── zip.rs              # ZIP implementation
│   │   └── tar.rs              # TAR implementation
│   ├── operations/
│   │   ├── mod.rs              # Operations module root
│   │   ├── compress.rs         # Compression operations
│   │   ├── decompress.rs       # Decompression operations
│   │   ├── collection.rs       # Multi-file collection
│   │   └── compound.rs         # Compound format handling
│   └── utils/
│       ├── mod.rs              # Utils module root
│       ├── logger.rs           # Logging infrastructure
│       ├── fs.rs               # File system utilities
│       ├── process.rs          # Process execution helpers
│       ├── timestamp.rs        # Timestamp generation
│       └── validation.rs       # Input validation
├── tests/
│   ├── integration_tests.rs    # Integration tests
│   └── fixtures/               # Test data
├── Cargo.toml                  # Dependencies and metadata
└── docs/
    ├── jcz_srs.md              # Requirements specification
    └── jcz_sdd.md              # This design document
```

---

## 3. Detailed Design

### 3.1 Core Module (`src/core/`)

#### 3.1.1 Compressor Trait (`src/core/compressor.rs`)

**Purpose**: Define common interface for all compression implementations.

```rust
use std::path::Path;
use crate::core::config::CompressionConfig;
use crate::core::error::JcResult;

/// Common interface for all compression/decompression implementations
pub trait Compressor: Send + Sync {
    /// Get the name of this compressor
    fn name(&self) -> &'static str;

    /// Get the file extension used by this compressor (e.g., "gz", "bz2")
    fn extension(&self) -> &'static str;

    /// Compress a single file
    fn compress(&self, input: &Path, config: &CompressionConfig) -> JcResult<PathBuf>;

    /// Decompress a single file
    fn decompress(&self, input: &Path, config: &CompressionConfig) -> JcResult<PathBuf>;

    /// Check if this compressor supports compression levels
    fn supports_levels(&self) -> bool;

    /// Validate compression level for this compressor
    fn validate_level(&self, level: u8) -> bool;

    /// Get default compression level
    fn default_level(&self) -> u8;
}

/// Extended trait for compressors that support multi-file operations
pub trait MultiFileCompressor: Compressor {
    /// Compress multiple files into a single archive
    fn compress_multi(
        &self,
        inputs: &[PathBuf],
        output_name: &str,
        config: &CompressionConfig,
    ) -> JcResult<PathBuf>;
}
```

**Design Rationale**:
- **Trait-based design**: Allows polymorphism and extensibility
- **Send + Sync bounds**: Enables safe concurrent usage
- **Result return types**: Idiomatic error handling
- **Configuration separation**: Config passed as parameter for flexibility
- **Extension trait**: `MultiFileCompressor` for TAR-specific functionality

#### 3.1.2 Configuration Structures (`src/core/config.rs`)

```rust
use std::path::PathBuf;

/// Timestamp formatting options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampOption {
    None,                    // 0: No timestamp
    Date,                    // 1: YYYYMMDD
    DateTime,                // 2: YYYYMMDD_HHMMSS
    Nanoseconds,             // 3: Nanoseconds only
}

impl TimestampOption {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TimestampOption::None),
            1 => Some(TimestampOption::Date),
            2 => Some(TimestampOption::DateTime),
            3 => Some(TimestampOption::Nanoseconds),
            _ => None,
        }
    }
}

/// Configuration for compression/decompression operations
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Compression level (0-9, meaning varies by algorithm)
    pub level: u8,

    /// Timestamp option for output filenames
    pub timestamp: TimestampOption,

    /// Destination directory for output files
    pub move_to: Option<PathBuf>,

    /// Show output file size (future feature)
    pub show_output_size: bool,

    /// Force overwrite without prompting during decompression
    pub force: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: 6,
            timestamp: TimestampOption::None,
            move_to: None,
            show_output_size: false,
            force: false,
        }
    }
}

impl CompressionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }

    pub fn with_timestamp(mut self, timestamp: TimestampOption) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn with_move_to(mut self, path: PathBuf) -> Self {
        self.move_to = Some(path);
        self
    }
}

/// Collection operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionMode {
    /// Include parent directory in archive (-a flag)
    WithParent,

    /// Archive files without parent directory wrapper (-A flag)
    Flat,
}

/// Configuration for collection operations (multi-file archives)
#[derive(Debug, Clone)]
pub struct CollectionConfig {
    /// Base configuration
    pub base: CompressionConfig,

    /// Package/archive name
    pub package_name: String,

    /// Collection mode
    pub mode: CollectionMode,
}
```

**Design Rationale**:
- **Enum for timestamp**: Type-safe options, exhaustive matching
- **Builder pattern**: Fluent configuration with `with_*` methods
- **Separation of concerns**: Collection config extends base config
- **Default trait**: Sensible defaults matching CLI defaults

#### 3.1.3 Error Handling (`src/core/error.rs`)

```rust
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Result type for JCZ operations
pub type JcResult<T> = Result<T, JcError>;

/// Comprehensive error type for JCZ operations
#[derive(Debug)]
pub enum JcError {
    /// File not found
    FileNotFound(PathBuf),

    /// Path is not a file (e.g., directory when file expected)
    NotAFile(PathBuf),

    /// Path is not a directory
    NotADirectory(PathBuf),

    /// Invalid file extension for operation
    InvalidExtension(PathBuf, String),

    /// Invalid compression level for algorithm
    InvalidCompressionLevel { algorithm: String, level: u8 },

    /// Invalid timestamp option
    InvalidTimestampOption(u8),

    /// Invalid compression command
    InvalidCommand(String),

    /// Duplicate basenames in collection
    DuplicateBasenames(Vec<String>),

    /// Archive/package name already exists
    NameExists(String),

    /// Move-to directory error
    MoveToError(String),

    /// Compression tool execution failed
    CompressionFailed {
        tool: String,
        stderr: String,
    },

    /// Decompression tool execution failed
    DecompressionFailed {
        tool: String,
        stderr: String,
    },

    /// I/O error
    Io(io::Error),

    /// Symbolic link resolution failed
    SymlinkResolution(PathBuf),

    /// Temporary directory creation failed
    TempDirFailed(String),

    /// No input files provided
    NoInputFiles,

    /// Generic error with message
    Other(String),
}

impl fmt::Display for JcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JcError::FileNotFound(path) => {
                write!(f, "File not found: {}", path.display())
            }
            JcError::NotAFile(path) => {
                write!(f, "{} is not a file", path.display())
            }
            JcError::NotADirectory(path) => {
                write!(f, "{} is not a directory", path.display())
            }
            JcError::InvalidExtension(path, expected) => {
                write!(f, "{} has invalid extension, expected: {}", path.display(), expected)
            }
            JcError::InvalidCompressionLevel { algorithm, level } => {
                write!(f, "Invalid compression level {} for {}", level, algorithm)
            }
            JcError::InvalidTimestampOption(opt) => {
                write!(f, "Invalid timestamp option: {}", opt)
            }
            JcError::InvalidCommand(cmd) => {
                write!(f, "Invalid compression command: {}", cmd)
            }
            JcError::DuplicateBasenames(names) => {
                write!(f, "Duplicate basenames in collection: {}", names.join(", "))
            }
            JcError::NameExists(name) => {
                write!(f, "{} already exists and cannot be used as package name", name)
            }
            JcError::MoveToError(msg) => {
                write!(f, "Move-to directory error: {}", msg)
            }
            JcError::CompressionFailed { tool, stderr } => {
                write!(f, "{} compression failed: {}", tool, stderr)
            }
            JcError::DecompressionFailed { tool, stderr } => {
                write!(f, "{} decompression failed: {}", tool, stderr)
            }
            JcError::Io(err) => {
                write!(f, "I/O error: {}", err)
            }
            JcError::SymlinkResolution(path) => {
                write!(f, "Failed to resolve symbolic link: {}", path.display())
            }
            JcError::TempDirFailed(msg) => {
                write!(f, "Temporary directory creation failed: {}", msg)
            }
            JcError::NoInputFiles => {
                write!(f, "No input files provided")
            }
            JcError::Other(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl std::error::Error for JcError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JcError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for JcError {
    fn from(err: io::Error) -> Self {
        JcError::Io(err)
    }
}
```

**Design Rationale**:
- **Rich error types**: Specific variants for each error condition
- **Display implementation**: User-friendly error messages
- **Error trait**: Standard Rust error handling
- **Type alias**: `JcResult<T>` for convenience
- **From trait**: Automatic conversion from `io::Error`

#### 3.1.4 Common Types (`src/core/types.rs`)

```rust
use std::path::PathBuf;

/// Compression format/algorithm identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionFormat {
    Gzip,
    Bzip2,
    Xz,
    Zip,
    Tar,
}

impl CompressionFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            CompressionFormat::Gzip => "gz",
            CompressionFormat::Bzip2 => "bz2",
            CompressionFormat::Xz => "xz",
            CompressionFormat::Zip => "zip",
            CompressionFormat::Tar => "tar",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "gz" => Some(CompressionFormat::Gzip),
            "bz2" => Some(CompressionFormat::Bzip2),
            "xz" => Some(CompressionFormat::Xz),
            "zip" => Some(CompressionFormat::Zip),
            "tar" => Some(CompressionFormat::Tar),
            _ => None,
        }
    }

    /// Get algorithm name
    pub fn name(&self) -> &'static str {
        match self {
            CompressionFormat::Gzip => "gzip",
            CompressionFormat::Bzip2 => "bzip2",
            CompressionFormat::Xz => "xz",
            CompressionFormat::Zip => "zip",
            CompressionFormat::Tar => "tar",
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
pub enum OperationMode {
    Compress,
    Decompress,
}

/// Validated input file information
#[derive(Debug, Clone)]
pub struct InputFile {
    /// Original path provided by user
    pub original_path: PathBuf,

    /// Resolved real path (after symlink resolution)
    pub real_path: PathBuf,

    /// File basename
    pub basename: String,

    /// Whether this was a symbolic link
    pub was_symlink: bool,
}
```

**Design Rationale**:
- **Enums for formats**: Type-safe format identification
- **Self-describing types**: Methods return relevant info about the type
- **Separation**: Simple vs. compound formats
- **InputFile struct**: Encapsulates validated file metadata

---

### 3.2 Compressor Implementations (`src/compressors/`)

#### 3.2.1 GZIP Implementation (`src/compressors/gzip.rs`)

```rust
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::core::compressor::Compressor;
use crate::core::config::CompressionConfig;
use crate::core::error::{JcError, JcResult};
use crate::utils::process::run_command_with_output;
use crate::utils::fs::{generate_output_filename, move_file_if_needed};
use crate::utils::logger::{debug, info};

/// GZIP compressor implementation
#[derive(Debug, Clone)]
pub struct GzipCompressor;

impl GzipCompressor {
    pub fn new() -> Self {
        Self
    }

    /// Validate that input is a file, not a directory
    fn validate_input(&self, path: &Path) -> JcResult<()> {
        if !path.exists() {
            return Err(JcError::FileNotFound(path.to_path_buf()));
        }

        if path.is_dir() {
            return Err(JcError::NotAFile(path.to_path_buf()));
        }

        Ok(())
    }
}

impl Compressor for GzipCompressor {
    fn name(&self) -> &'static str {
        "gzip"
    }

    fn extension(&self) -> &'static str {
        "gz"
    }

    fn compress(&self, input: &Path, config: &CompressionConfig) -> JcResult<PathBuf> {
        self.validate_input(input)?;

        let output_path = generate_output_filename(input, "gz", config.timestamp)?;
        info!("Compressing {} to {} with gzip", input.display(), output_path.display());
        debug!("Compression level: {}", config.level);

        // Create output file with buffered writer
        let output_file = File::create(&output_path)?;
        let mut writer = BufWriter::new(output_file);

        // Execute gzip command
        let mut cmd = Command::new("gzip");
        cmd.arg(format!("-{}", config.level))
           .arg("--keep")
           .arg("--stdout")
           .arg(input)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        debug!("Executing: {:?}", cmd);

        let mut child = cmd.spawn()
            .map_err(|e| JcError::Other(format!("Failed to spawn gzip: {}", e)))?;

        // Stream stdout to output file
        if let Some(mut stdout) = child.stdout.take() {
            std::io::copy(&mut stdout, &mut writer)?;
        }

        writer.flush()?;

        // Wait for process and check exit status
        let output = child.wait_with_output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JcError::CompressionFailed {
                tool: "gzip".to_string(),
                stderr: stderr.to_string(),
            });
        }

        // Move to destination if specified
        let final_path = move_file_if_needed(&output_path, &config.move_to)?;

        info!("Compressed file: {}", final_path.display());
        Ok(final_path)
    }

    fn decompress(&self, input: &Path, config: &CompressionConfig) -> JcResult<PathBuf> {
        // Validate extension
        if !input.to_string_lossy().ends_with(".gz") {
            return Err(JcError::InvalidExtension(
                input.to_path_buf(),
                "gz".to_string(),
            ));
        }

        debug!("Decompressing {} with gzip", input.display());

        // Execute gzip decompression
        let mut cmd = Command::new("gzip");
        cmd.arg("-d")
           .arg("-k")
           .arg(input);

        let output = cmd.output()
            .map_err(|e| JcError::Other(format!("Failed to execute gzip: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JcError::DecompressionFailed {
                tool: "gzip".to_string(),
                stderr: stderr.to_string(),
            });
        }

        // Determine output filename (remove .gz)
        let output_path = input.with_extension("");

        // Move to destination if specified
        let final_path = move_file_if_needed(&output_path, &config.move_to)?;

        info!("Decompressed file: {}", final_path.display());
        Ok(final_path)
    }

    fn supports_levels(&self) -> bool {
        true
    }

    fn validate_level(&self, level: u8) -> bool {
        (1..=9).contains(&level)
    }

    fn default_level(&self) -> u8 {
        6
    }
}
```

**Additional Methods for Isolated Decompression**:

Each compressor also implements a `decompress_in_dir` method for working directory isolation:

```rust
impl GzipCompressor {
    pub fn decompress_in_dir(
        &self,
        input: &Path,
        working_dir: &Path,
        _config: &CompressionConfig,
    ) -> JcResult<PathBuf> {
        // Copy input file to working directory
        let work_input = copy_to_dir(input, working_dir)?;

        // Execute gzip decompression with -f flag to force overwrite
        let mut cmd = Command::new("gzip");
        cmd.arg("-d").arg("-f").arg(&work_input);

        let output = cmd.output()
            .map_err(|e| JcError::Other(format!("Failed to execute gzip: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JcError::DecompressionFailed {
                tool: "gzip".to_string(),
                stderr: stderr.to_string(),
            });
        }

        let output_path = work_input.with_extension("");
        Ok(output_path)
    }
}
```

**Design Rationale**:
- **Streaming I/O**: Use `stdout` pipe to stream compressed data
- **Buffered writes**: Efficient I/O with `BufWriter`
- **Error handling**: Convert process errors to `JcError`
- **Validation**: Check input before processing
- **Logging**: Debug and info messages for visibility
- **Isolated Decompression**: `decompress_in_dir` operates in temporary directory to prevent conflicts
- **Force Overwrite**: Uses `-f` flag to overwrite files in working directory without prompting

#### 3.2.2 TAR Implementation (`src/compressors/tar.rs`)

```rust
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::compressor::{Compressor, MultiFileCompressor};
use crate::core::config::CompressionConfig;
use crate::core::error::{JcError, JcResult};
use crate::utils::fs::{generate_output_filename, move_file_if_needed};
use crate::utils::logger::{debug, info};

/// TAR archiver implementation
#[derive(Debug, Clone)]
pub struct TarCompressor;

impl TarCompressor {
    pub fn new() -> Self {
        Self
    }
}

impl Compressor for TarCompressor {
    fn name(&self) -> &'static str {
        "tar"
    }

    fn extension(&self) -> &'static str {
        "tar"
    }

    fn compress(&self, input: &Path, config: &CompressionConfig) -> JcResult<PathBuf> {
        if !input.exists() {
            return Err(JcError::FileNotFound(input.to_path_buf()));
        }

        let output_path = generate_output_filename(input, "tar", config.timestamp)?;
        info!("Creating TAR archive {} from {}", output_path.display(), input.display());

        // Get parent directory and basename
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        let basename = input.file_name()
            .ok_or_else(|| JcError::Other("Invalid filename".to_string()))?;

        // Build tar command
        let mut cmd = Command::new("tar");

        if parent != Path::new(".") {
            cmd.arg("-C").arg(parent);
        }

        cmd.arg("-cf")
           .arg(&output_path)
           .arg(basename);

        debug!("Executing: {:?}", cmd);

        let output = cmd.output()
            .map_err(|e| JcError::Other(format!("Failed to execute tar: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JcError::CompressionFailed {
                tool: "tar".to_string(),
                stderr: stderr.to_string(),
            });
        }

        // Move to destination if specified
        let final_path = move_file_if_needed(&output_path, &config.move_to)?;

        info!("Created TAR archive: {}", final_path.display());
        Ok(final_path)
    }

    fn decompress(&self, input: &Path, config: &CompressionConfig) -> JcResult<PathBuf> {
        if !input.to_string_lossy().ends_with(".tar") {
            return Err(JcError::InvalidExtension(
                input.to_path_buf(),
                "tar".to_string(),
            ));
        }

        debug!("Extracting TAR archive {}", input.display());

        let parent = input.parent().unwrap_or_else(|| Path::new("."));

        let mut cmd = Command::new("tar");
        cmd.arg("-x")
           .arg("-C").arg(parent)
           .arg("-f").arg(input);

        let output = cmd.output()
            .map_err(|e| JcError::Other(format!("Failed to execute tar: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JcError::DecompressionFailed {
                tool: "tar".to_string(),
                stderr: stderr.to_string(),
            });
        }

        // Output is the filename without .tar extension
        let output_path = input.with_extension("");

        // Move to destination if specified
        let final_path = move_file_if_needed(&output_path, &config.move_to)?;

        info!("Extracted TAR archive to: {}", final_path.display());
        Ok(final_path)
    }

    fn supports_levels(&self) -> bool {
        false // TAR doesn't support compression levels
    }

    fn validate_level(&self, _level: u8) -> bool {
        true // Always valid (no-op)
    }

    fn default_level(&self) -> u8 {
        0
    }
}

impl MultiFileCompressor for TarCompressor {
    fn compress_multi(
        &self,
        inputs: &[PathBuf],
        output_name: &str,
        config: &CompressionConfig,
    ) -> JcResult<PathBuf> {
        if inputs.is_empty() {
            return Err(JcError::NoInputFiles);
        }

        let output_path = PathBuf::from(output_name)
            .with_extension("tar");

        info!("Creating multi-file TAR archive: {}", output_path.display());

        // Find common parent directory
        // For simplicity, we'll use current directory and provide relative paths
        let mut cmd = Command::new("tar");
        cmd.arg("-cf").arg(&output_path);

        for input in inputs {
            // Get basename for flat archiving
            let basename = input.file_name()
                .ok_or_else(|| JcError::Other("Invalid filename".to_string()))?;
            cmd.arg(basename);
        }

        debug!("Executing: {:?}", cmd);

        let output = cmd.output()
            .map_err(|e| JcError::Other(format!("Failed to execute tar: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JcError::CompressionFailed {
                tool: "tar".to_string(),
                stderr: stderr.to_string(),
            });
        }

        // Move to destination if specified
        let final_path = move_file_if_needed(&output_path, &config.move_to)?;

        Ok(final_path)
    }
}
```

**Design Rationale**:
- **Directory handling**: TAR can archive both files and directories
- **MultiFileCompressor**: Separate trait for multi-file capability
- **Path manipulation**: Handle parent/basename splitting
- **No compression level**: Returns true for any level (no-op)

#### 3.2.3 ZIP Implementation (`src/compressors/zip.rs`)

**Purpose**: ZIP compression and decompression using system `zip` and `unzip` commands.

**Key Features**:
- Supports both files and directories
- Archive format (can contain multiple files)
- Compression levels 0-9
- Cross-platform compatibility

**Design Rationale**:
- **Archive support**: Unlike gzip/bzip2/xz, ZIP is both a compression algorithm and archive format
- **Directory compression**: Recursive flag (`-r`) enables directory compression
- **Quiet mode**: Suppress verbose output for cleaner logs
- **Flexible extraction**: Handles single files, directories, and multiple loose files

**Implementation Notes**:
- Uses `zip` command for compression with `-q` (quiet) and optional `-r` (recursive)
- Uses `unzip` command for decompression with `-o` (overwrite)
- Compression levels: 0 (store only) to 9 (maximum compression)
- Default level: 6
- Supports timestamp options for output naming
- `decompress_in_dir` method detects extraction result (single file, directory, or multiple files)

#### 3.2.4 Compressor Factory (`src/compressors/mod.rs`)

```rust
pub mod gzip;
pub mod bzip2;
pub mod xz;
pub mod zip;
pub mod tar;

use crate::core::compressor::Compressor;
use crate::core::types::CompressionFormat;
use crate::core::error::{JcError, JcResult};

/// Create a compressor instance for the given format
pub fn create_compressor(format: CompressionFormat) -> Box<dyn Compressor> {
    match format {
        CompressionFormat::Gzip => Box::new(gzip::GzipCompressor::new()),
        CompressionFormat::Bzip2 => Box::new(bzip2::Bzip2Compressor::new()),
        CompressionFormat::Xz => Box::new(xz::XzCompressor::new()),
        CompressionFormat::Zip => Box::new(zip::ZipCompressor::new()),
        CompressionFormat::Tar => Box::new(tar::TarCompressor::new()),
    }
}

/// Detect compression format from file extension
pub fn detect_format(path: &Path) -> Option<CompressionFormat> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(CompressionFormat::from_extension)
}
```

---

### 3.3 Operations Module (`src/operations/`)

#### 3.3.1 Compression Operations (`src/operations/compress.rs`)

```rust
use std::path::PathBuf;
use rayon::prelude::*;

use crate::core::config::CompressionConfig;
use crate::core::types::CompressionFormat;
use crate::core::error::{JcError, JcResult};
use crate::compressors::create_compressor;
use crate::utils::logger::{error, info};

/// Compress a single file
pub fn compress_file(
    input: &PathBuf,
    format: CompressionFormat,
    config: &CompressionConfig,
) -> JcResult<PathBuf> {
    let compressor = create_compressor(format);

    // Validate compression level if supported
    if compressor.supports_levels() && !compressor.validate_level(config.level) {
        return Err(JcError::InvalidCompressionLevel {
            algorithm: compressor.name().to_string(),
            level: config.level,
        });
    }

    compressor.compress(input, config)
}

/// Compress multiple files concurrently
pub fn compress_files(
    inputs: Vec<PathBuf>,
    format: CompressionFormat,
    config: CompressionConfig,
) -> Vec<JcResult<PathBuf>> {
    info!("Compressing {} files with {}", inputs.len(), format.name());

    // Use rayon for parallel processing
    inputs.par_iter()
        .map(|input| {
            match compress_file(input, format, &config) {
                Ok(output) => Ok(output),
                Err(e) => {
                    error!("Failed to compress {}: {}", input.display(), e);
                    Err(e)
                }
            }
        })
        .collect()
}
```

**Design Rationale**:
- **Rayon for parallelism**: Efficient work-stealing parallelism
- **Error collection**: Return all results, including failures
- **Logging**: Log individual file failures
- **Configuration cloning**: Safe to pass config to parallel tasks

#### 3.3.2 Decompression Operations (`src/operations/decompress.rs`)

```rust
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;

use crate::compressors::{detect_format, Bzip2Compressor, GzipCompressor, TarCompressor, XzCompressor};
use crate::core::config::CompressionConfig;
use crate::core::error::{JcError, JcResult};
use crate::utils::{create_decompress_temp_dir, debug, error, info, prompt_overwrite};

/// Helper function to decompress in a working directory based on format
fn decompress_in_working_dir(
    format: crate::core::types::CompressionFormat,
    input: &PathBuf,
    working_dir: &PathBuf,
    config: &CompressionConfig,
) -> JcResult<PathBuf> {
    use crate::core::types::CompressionFormat;

    match format {
        CompressionFormat::Gzip => {
            let compressor = GzipCompressor::new();
            compressor.decompress_in_dir(input, working_dir, config)
        }
        CompressionFormat::Bzip2 => {
            let compressor = Bzip2Compressor::new();
            compressor.decompress_in_dir(input, working_dir, config)
        }
        CompressionFormat::Xz => {
            let compressor = XzCompressor::new();
            compressor.decompress_in_dir(input, working_dir, config)
        }
        CompressionFormat::Tar => {
            let compressor = TarCompressor::new();
            compressor.decompress_in_dir(input, working_dir, config)
        }
    }
}

/// Decompress a single file, handling compound formats
pub fn decompress_file(input: &PathBuf, config: &CompressionConfig) -> JcResult<PathBuf> {
    // Create a temporary directory for decompression work
    let temp_dir = create_decompress_temp_dir()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    debug!("Created temp directory: {}", temp_dir_path.display());

    let mut current_file = input.clone();
    let mut current_in_temp = false;

    // Iteratively decompress until no more compression detected
    loop {
        let format = detect_format(&current_file).ok_or_else(|| {
            JcError::InvalidExtension(
                current_file.clone(),
                "supported compression format".to_string(),
            )
        })?;

        info!(
            "Decompression iteration: format={:?}, current_file={}",
            format,
            current_file.display()
        );

        // Decompress in temp directory
        let output = decompress_in_working_dir(format, &current_file, &temp_dir_path, config)?;

        info!("Decompressed to: {}", output.display());

        current_file = output;
        current_in_temp = true;

        // Check if output has another compression layer
        if detect_format(&current_file).is_none() {
            info!("No more compression layers detected");
            break;
        }
    }

    // Determine final destination
    let final_dest = if let Some(ref move_to) = config.move_to {
        if current_file.is_dir() && current_file == temp_dir_path {
            // Multiple files extracted - put them directly in move_to
            move_to.clone()
        } else {
            // Single file or directory - create subdirectory
            let mut dest = input.clone();
            while detect_format(&dest).is_some() {
                dest = dest.with_extension("");
            }
            let filename = dest
                .file_name()
                .ok_or_else(|| JcError::Other("Invalid output filename".to_string()))?;
            move_to.join(filename)
        }
    } else {
        // Determine output based on input filename
        let mut dest = input.clone();
        // Remove all compression extensions
        while detect_format(&dest).is_some() {
            dest = dest.with_extension("");
        }
        dest
    };

    debug!("Final destination: {}", final_dest.display());

    // Move from temp directory to final destination
    if current_in_temp {
        if current_file.is_dir() {
            if current_file == temp_dir_path {
                // Multiple loose files from TAR - copy contents directly
                fs::create_dir_all(&final_dest).map_err(|e| JcError::Io(e))?;
                for entry in fs::read_dir(&current_file).map_err(|e| JcError::Io(e))? {
                    let entry = entry.map_err(|e| JcError::Io(e))?;
                    let src_path = entry.path();
                    let dst_path = final_dest.join(entry.file_name());

                    // Check if individual file exists and prompt for overwrite
                    if dst_path.exists() && !config.force {
                        if !prompt_overwrite(&dst_path)? {
                            info!("Skipping {}", dst_path.display());
                            continue;
                        }
                    }

                    use crate::utils::copy_recursive;
                    if src_path.is_dir() {
                        copy_recursive(&src_path, &dst_path).map_err(|e| JcError::Io(e))?;
                    } else {
                        fs::copy(&src_path, &dst_path).map_err(|e| JcError::Io(e))?;
                    }
                }
                info!("Decompressed {} files to: {}",
                      fs::read_dir(&final_dest).map(|d| d.count()).unwrap_or(0),
                      final_dest.display());
            } else {
                // Subdirectory extracted from TAR
                if final_dest.exists() && !config.force {
                    if !prompt_overwrite(&final_dest)? {
                        return Err(JcError::Other(format!(
                            "Decompression aborted: directory already exists: {}",
                            final_dest.display()
                        )));
                    }
                }
                use crate::utils::copy_recursive;
                copy_recursive(&current_file, &final_dest).map_err(|e| JcError::Io(e))?;
                info!("Decompressed directory: {}", final_dest.display());
            }
        } else {
            // Copy single file
            if final_dest.exists() && !config.force {
                if !prompt_overwrite(&final_dest)? {
                    return Err(JcError::Other(format!(
                        "Decompression aborted: file already exists: {}",
                        final_dest.display()
                    )));
                }
            }
            fs::copy(&current_file, &final_dest).map_err(|e| JcError::Io(e))?;
            info!("Decompressed file: {}", final_dest.display());
        }
    }

    // temp_dir will be automatically cleaned up when it goes out of scope
    Ok(final_dest)
}

/// Decompress multiple files concurrently
pub fn decompress_files(inputs: Vec<PathBuf>, config: CompressionConfig) -> Vec<JcResult<PathBuf>> {
    info!("Decompressing {} files", inputs.len());

    inputs
        .par_iter()
        .map(|input| match decompress_file(input, &config) {
            Ok(output) => Ok(output),
            Err(e) => {
                error!("Failed to decompress {}: {}", input.display(), e);
                Err(e)
            }
        })
        .collect()
}
```

**Design Rationale**:
- **Temporary Directory Isolation**: All decompression work happens in isolated /tmp directories
  - Uses `tempfile::TempDir` for automatic RAII-based cleanup
  - Prevents conflicts with existing files in working directory
  - Each decompression gets unique temporary directory
  - Cleanup happens automatically even on panic or early return
- **Iterative Decompression**: Handle compound formats (tar.gz, tar.bz2, tar.xz)
  - Detects compression layers from file extension
  - Decompresses each layer in sequence within same temp directory
  - Continues until no more compression detected
- **Multi-File Archive Support**: Correctly handles TAR archives with multiple files
  - Detects single file vs multiple files vs single directory
  - For multiple files with -C flag: extracts directly to destination
  - For multiple files without -C: creates subdirectory named after archive
  - For single files: always extracts to filename without extensions
- **Overwrite Protection with Force Flag**: Interactive prompts unless --force/-f specified
  - Multi-file extraction: prompts for each individual file
  - Single file extraction: prompts once before extraction
  - User can skip files by responding 'n'
  - Skipped files are logged for transparency
- **Format Detection**: Automatic detection from file extension
- **Parallel Processing**: Use rayon for concurrent decompression of multiple files
- **Error Handling**: Comprehensive error messages for all failure scenarios

#### 3.3.3 Compound Format Handling (`src/operations/compound.rs`)

```rust
use std::path::PathBuf;

use crate::core::config::CompressionConfig;
use crate::core::types::CompoundFormat;
use crate::core::error::{JcError, JcResult};
use crate::compressors::create_compressor;
use crate::utils::logger::{debug, info};
use crate::utils::fs;

/// Compress file(s) with compound format (TAR + secondary compression)
pub fn compress_compound(
    input: &PathBuf,
    format: CompoundFormat,
    config: &CompressionConfig,
) -> JcResult<PathBuf> {
    info!("Compressing {} with compound format: {}", input.display(), format.extension());

    // Step 1: Create TAR archive
    let tar_compressor = create_compressor(format.primary());
    let tar_config = CompressionConfig {
        level: 0, // TAR doesn't use compression level
        timestamp: config.timestamp,
        move_to: None, // Don't move intermediate file
        show_output_size: false,
    };

    let tar_output = tar_compressor.compress(input, &tar_config)?;
    debug!("Created intermediate TAR: {}", tar_output.display());

    // Step 2: Compress TAR with secondary compressor
    let secondary_compressor = create_compressor(format.secondary());
    let secondary_output = secondary_compressor.compress(&tar_output, config)?;

    // Step 3: Remove intermediate TAR file
    if let Err(e) = fs::remove_file_silent(&tar_output) {
        debug!("Failed to remove intermediate TAR: {}", e);
    }

    info!("Created compound archive: {}", secondary_output.display());
    Ok(secondary_output)
}

/// Compress multiple files with compound format
pub fn compress_compound_batch(
    inputs: Vec<PathBuf>,
    format: CompoundFormat,
    config: CompressionConfig,
) -> Vec<JcResult<PathBuf>> {
    use rayon::prelude::*;

    inputs.par_iter()
        .map(|input| compress_compound(input, format, &config))
        .collect()
}
```

#### 3.3.4 Collection Operations (`src/operations/collection.rs`)

```rust
use std::path::{Path, PathBuf};
use std::fs;

use crate::core::config::{CollectionConfig, CollectionMode, CompressionConfig};
use crate::core::types::CompoundFormat;
use crate::core::error::{JcError, JcResult};
use crate::compressors::create_compressor;
use crate::utils::logger::{debug, info};
use crate::utils::fs as fsutil;

/// Collect multiple files into a compressed archive
pub fn collect_and_compress(
    inputs: Vec<PathBuf>,
    format: CompoundFormat,
    collection_config: CollectionConfig,
) -> JcResult<PathBuf> {
    // Validate inputs
    if inputs.is_empty() {
        return Err(JcError::NoInputFiles);
    }

    // Check for duplicate basenames
    let basenames: Vec<String> = inputs.iter()
        .filter_map(|p| p.file_name())
        .filter_map(|n| n.to_str())
        .map(|s| s.to_string())
        .collect();

    let mut unique_basenames = basenames.clone();
    unique_basenames.sort();
    unique_basenames.dedup();

    if basenames.len() != unique_basenames.len() {
        let duplicates: Vec<String> = basenames.iter()
            .filter(|name| basenames.iter().filter(|n| *n == *name).count() > 1)
            .cloned()
            .collect();
        return Err(JcError::DuplicateBasenames(duplicates));
    }

    // Check package name doesn't exist
    let package_path = PathBuf::from(&collection_config.package_name);
    if package_path.exists() {
        return Err(JcError::NameExists(collection_config.package_name.clone()));
    }

    info!("Collecting {} files into {}", inputs.len(), collection_config.package_name);

    // Create temporary staging directory
    let temp_dir = fsutil::create_temp_dir("jczpkg_")?;
    debug!("Created temporary directory: {}", temp_dir.display());

    // Ensure cleanup on exit
    let _cleanup = CleanupGuard::new(temp_dir.clone());

    let staging_dir = match collection_config.mode {
        CollectionMode::WithParent => {
            // Create subdirectory with package name
            let pkg_dir = temp_dir.join(&collection_config.package_name);
            fs::create_dir(&pkg_dir)?;
            pkg_dir
        }
        CollectionMode::Flat => {
            // Use temp dir directly
            temp_dir.clone()
        }
    };

    // Copy files to staging directory
    for input in &inputs {
        let basename = input.file_name()
            .ok_or_else(|| JcError::Other("Invalid filename".to_string()))?;
        let dest = staging_dir.join(basename);

        debug!("Copying {} to {}", input.display(), dest.display());
        fsutil::copy_recursive(input, &dest)?;
    }

    // Create TAR archive
    let tar_compressor = create_compressor(format.primary());

    let archive_input = match collection_config.mode {
        CollectionMode::WithParent => {
            // Archive the package directory
            temp_dir.join(&collection_config.package_name)
        }
        CollectionMode::Flat => {
            // Archive contents directly
            staging_dir
        }
    };

    let tar_config = CompressionConfig {
        level: 0,
        timestamp: collection_config.base.timestamp,
        move_to: None,
        show_output_size: false,
    };

    // Generate TAR filename
    let tar_filename = if collection_config.mode == CollectionMode::Flat {
        // For flat mode, create TAR from staging dir contents
        use crate::core::compressor::MultiFileCompressor;
        let tar_multi = tar_compressor.as_any()
            .downcast_ref::<crate::compressors::tar::TarCompressor>()
            .ok_or_else(|| JcError::Other("Expected TAR compressor".to_string()))?;

        let file_list: Vec<PathBuf> = inputs.iter().cloned().collect();
        tar_multi.compress_multi(&file_list, &collection_config.package_name, &tar_config)?
    } else {
        tar_compressor.compress(&archive_input, &tar_config)?
    };

    debug!("Created TAR archive: {}", tar_filename.display());

    // Apply secondary compression
    let final_output = if format != CompoundFormat::Tgz {
        let secondary_compressor = create_compressor(format.secondary());
        let compressed = secondary_compressor.compress(&tar_filename, &collection_config.base)?;

        // Remove intermediate TAR
        fsutil::remove_file_silent(&tar_filename)?;

        compressed
    } else {
        tar_filename
    };

    // Move to destination or current directory
    let destination = collection_config.base.move_to
        .unwrap_or_else(|| PathBuf::from("."));

    let final_path = fsutil::move_file(&final_output, &destination)?;

    info!("Created collection archive: {}", final_path.display());
    Ok(final_path)
}

/// RAII guard for cleaning up temporary directory
struct CleanupGuard {
    path: PathBuf,
}

impl CleanupGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_dir_all(&self.path) {
            debug!("Failed to cleanup temp directory {}: {}", self.path.display(), e);
        } else {
            debug!("Cleaned up temp directory: {}", self.path.display());
        }
    }
}
```

**Design Rationale**:
- **RAII cleanup**: `CleanupGuard` ensures temp directory removal
- **Duplicate detection**: Validate no basename collisions
- **Two-mode support**: WithParent vs. Flat archiving
- **Staging directory**: Isolate collection process

---

### 3.4 Utility Module (`src/utils/`)

#### 3.4.1 Logging (`src/utils/logger.rs`)

```rust
use std::sync::OnceLock;
use env_logger::{Builder, Env};
use log::{Level, LevelFilter};

static LOGGER_INIT: OnceLock<()> = OnceLock::new();

/// Initialize the logging system based on JCZDBG environment variable
pub fn init_logger() {
    LOGGER_INIT.get_or_init(|| {
        let env = Env::default()
            .filter_or("JCZDBG", "info")
            .write_style("JCZDBG_STYLE");

        let mut builder = Builder::from_env(env);

        // Map JCZDBG values to log levels
        let level = std::env::var("JCZDBG")
            .ok()
            .and_then(|val| match val.to_lowercase().as_str() {
                "error" => Some(LevelFilter::Error),
                "warn" => Some(LevelFilter::Warn),
                "info" => Some(LevelFilter::Info),
                "debug" => Some(LevelFilter::Debug),
                _ => None,
            })
            .unwrap_or(LevelFilter::Info);

        builder
            .filter_level(level)
            .format_module_path(false)
            .format_target(false)
            .init();
    });
}

// Re-export log macros for convenience
pub use log::{debug, error, info, warn};
```

**Design Rationale**:
- **env_logger crate**: Standard Rust logging
- **JCZDBG variable**: Match original behavior
- **OnceLock**: Thread-safe initialization
- **Re-export macros**: Convenient imports

#### 3.4.2 File System Utilities (`src/utils/fs.rs`)

```rust
use std::path::{Path, PathBuf};
use std::fs;
use std::io;

use crate::core::config::TimestampOption;
use crate::core::error::{JcError, JcResult};
use crate::utils::timestamp::generate_timestamp;

/// Generate output filename with optional timestamp
pub fn generate_output_filename(
    input: &Path,
    extension: &str,
    timestamp_opt: TimestampOption,
) -> JcResult<PathBuf> {
    let mut filename = input.as_os_str().to_string_lossy().to_string();

    // Remove trailing slash if present
    if filename.ends_with('/') {
        filename.pop();
    }

    // Add timestamp if requested
    if timestamp_opt != TimestampOption::None {
        let ts = generate_timestamp(timestamp_opt);
        filename.push('_');
        filename.push_str(&ts);
    }

    // Add extension
    filename.push('.');
    filename.push_str(extension);

    Ok(PathBuf::from(filename))
}

/// Move file to destination directory if specified
pub fn move_file_if_needed(
    source: &Path,
    move_to: &Option<PathBuf>,
) -> JcResult<PathBuf> {
    if let Some(dest_dir) = move_to {
        move_file(source, dest_dir)
    } else {
        Ok(source.to_path_buf())
    }
}

/// Move file to destination directory
pub fn move_file(source: &Path, dest_dir: &Path) -> JcResult<PathBuf> {
    // Validate destination is a directory
    if !dest_dir.is_dir() {
        return Err(JcError::NotADirectory(dest_dir.to_path_buf()));
    }

    let filename = source.file_name()
        .ok_or_else(|| JcError::Other("Invalid source filename".to_string()))?;

    let dest_path = dest_dir.join(filename);

    fs::rename(source, &dest_path)?;

    Ok(dest_path)
}

/// Recursively copy file or directory
pub fn copy_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            copy_recursive(&src_path, &dst_path)?;
        }
    } else {
        fs::copy(src, dst)?;
    }
    Ok(())
}

/// Remove file, ignoring errors
pub fn remove_file_silent(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// Create temporary directory with prefix
pub fn create_temp_dir(prefix: &str) -> JcResult<PathBuf> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let dir_name = format!("{}{:x}", prefix, timestamp);
    let temp_path = PathBuf::from(dir_name);

    fs::create_dir(&temp_path)
        .map_err(|e| JcError::TempDirFailed(e.to_string()))?;

    Ok(temp_path)
}

/// Create a temporary directory for decompression work in /tmp
/// Returns a TempDir that will be automatically cleaned up when dropped (RAII)
pub fn create_decompress_temp_dir() -> JcResult<TempDir> {
    use tempfile::TempDir;

    TempDir::new_in("/tmp")
        .map_err(|e| JcError::TempDirFailed(format!("Failed to create temp directory: {}", e)))
}

/// Copy a file to a target directory, preserving the filename
pub fn copy_to_dir(source: &Path, target_dir: &Path) -> JcResult<PathBuf> {
    let filename = source
        .file_name()
        .ok_or_else(|| JcError::Other("Invalid source filename".to_string()))?;

    let dest_path = target_dir.join(filename);

    // Check if source and destination are the same file
    if source.canonicalize().ok() == dest_path.canonicalize().ok() {
        // File is already in the target directory, no need to copy
        return Ok(dest_path);
    }

    fs::copy(source, &dest_path)
        .map_err(|e| JcError::Io(e))?;

    Ok(dest_path)
}
```

**Key Features**:
- **RAII Temporary Directories**: `create_decompress_temp_dir` uses `tempfile::TempDir` for automatic cleanup
- **Same-File Detection**: `copy_to_dir` checks if source and destination are the same to avoid unnecessary copying
- **Recursive Copy**: `copy_recursive` handles both files and directories
- **Error Handling**: All functions return `JcResult` for consistent error propagation

#### 3.4.3 User Prompt Utilities (`src/utils/prompt.rs`)

```rust
use std::io::{self, Write};
use std::path::Path;

use crate::core::error::{JcError, JcResult};

/// Prompt user for overwrite confirmation
pub fn prompt_overwrite(file_path: &Path) -> JcResult<bool> {
    print!("File '{}' already exists. Overwrite? (y/n): ", file_path.display());
    io::stdout().flush().map_err(|e| JcError::Io(e))?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| JcError::Io(e))?;

    let response = input.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}
```

**Key Features**:
- **Interactive Prompts**: Ask user for confirmation before overwriting files
- **Case-Insensitive**: Accepts both 'y'/'Y' and 'yes'/'YES'
- **Error Handling**: Proper handling of I/O errors
- **User-Friendly**: Clear prompt message showing file path

#### 3.4.4 Timestamp Generation (`src/utils/timestamp.rs`)

```rust
use chrono::Local;
use crate::core::config::TimestampOption;

/// Generate timestamp string based on option
pub fn generate_timestamp(option: TimestampOption) -> String {
    let now = Local::now();

    match option {
        TimestampOption::None => String::new(),
        TimestampOption::Date => {
            now.format("%Y%m%d").to_string()
        }
        TimestampOption::DateTime => {
            now.format("%Y%m%d_%H%M%S").to_string()
        }
        TimestampOption::Nanoseconds => {
            now.timestamp_subsec_nanos().to_string()
        }
    }
}
```

#### 3.4.5 Input Validation (`src/utils/validation.rs`)

```rust
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::{HashMap, HashSet};

use crate::core::types::InputFile;
use crate::core::error::{JcError, JcResult};
use crate::utils::logger::debug;

/// Validate and process input files
pub fn validate_input_files(paths: Vec<PathBuf>) -> JcResult<Vec<InputFile>> {
    if paths.is_empty() {
        return Err(JcError::NoInputFiles);
    }

    let mut validated = Vec::new();
    let mut seen_paths = HashSet::new();
    let mut basenames: HashMap<String, usize> = HashMap::new();

    for path in paths {
        // Check if file exists
        let metadata = fs::metadata(&path)
            .map_err(|_| JcError::FileNotFound(path.clone()))?;

        // Resolve symbolic links
        let (real_path, was_symlink) = if metadata.file_type().is_symlink() {
            debug!("{} is a symbolic link, resolving", path.display());
            let real = resolve_symlink(&path)?;
            (real, true)
        } else {
            (path.clone(), false)
        };

        // Check for duplicates
        if !seen_paths.insert(real_path.clone()) {
            debug!("Skipping duplicate path: {}", real_path.display());
            continue;
        }

        // Get basename
        let basename = real_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| JcError::Other("Invalid filename".to_string()))?
            .to_string();

        // Track basename occurrences
        *basenames.entry(basename.clone()).or_insert(0) += 1;

        validated.push(InputFile {
            original_path: path,
            real_path,
            basename,
            was_symlink,
        });
    }

    Ok(validated)
}

/// Check if there are duplicate basenames
pub fn check_duplicate_basenames(files: &[InputFile]) -> Option<Vec<String>> {
    let mut basename_counts: HashMap<&str, usize> = HashMap::new();

    for file in files {
        *basename_counts.entry(&file.basename).or_insert(0) += 1;
    }

    let duplicates: Vec<String> = basename_counts.iter()
        .filter(|(_, &count)| count > 1)
        .map(|(name, _)| name.to_string())
        .collect();

    if duplicates.is_empty() {
        None
    } else {
        Some(duplicates)
    }
}

/// Resolve symbolic link to real path
fn resolve_symlink(path: &Path) -> JcResult<PathBuf> {
    use std::process::Command;

    let output = Command::new("readlink")
        .arg("-f")
        .arg(path)
        .output()
        .map_err(|_| JcError::SymlinkResolution(path.to_path_buf()))?;

    if !output.status.success() {
        return Err(JcError::SymlinkResolution(path.to_path_buf()));
    }

    let real_path = String::from_utf8_lossy(&output.stdout);
    let real_path = real_path.trim();

    Ok(PathBuf::from(real_path))
}

/// Validate destination directory (creates it if it doesn't exist)
pub fn validate_move_to(path: &Path) -> JcResult<()> {
    if !path.exists() {
        // Create the directory if it doesn't exist
        fs::create_dir_all(path)
            .map_err(|e| JcError::MoveToError(format!("Failed to create directory: {}", e)))?;
    }

    if !path.is_dir() {
        return Err(JcError::NotADirectory(path.to_path_buf()));
    }

    // Check if writable by attempting to create a test file
    // (More robust than checking permissions)
    let test_file = path.join(".jcz_write_test");
    match fs::File::create(&test_file) {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
            Ok(())
        }
        Err(_) => {
            Err(JcError::MoveToError("Directory is not writable".to_string()))
        }
    }
}
```

---

### 3.5 CLI Module (`src/cli/`)

#### 3.5.1 Argument Parsing (`src/cli/args.rs`)

```rust
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "jcz")]
#[command(author = "JCZ Contributors")]
#[command(version)]
#[command(about = "Just Compress Zip - A unified compression utility", long_about = None)]
pub struct CliArgs {
    /// Decompress mode
    #[arg(short = 'd', long)]
    pub decompress: bool,

    /// Compression command
    #[arg(short = 'c', long, default_value = "tgz")]
    pub command: String,

    /// Compression level (1-9)
    #[arg(short = 'l', long, default_value = "6")]
    pub level: u8,

    /// Move compressed file to specified directory
    #[arg(short = 'C', long)]
    pub move_to: Option<PathBuf>,

    /// Collect files into archive (with parent directory)
    #[arg(short = 'a', long)]
    pub collect: Option<String>,

    /// Collect files into archive (flat, without parent directory)
    #[arg(short = 'A', long)]
    pub collect_flat: Option<String>,

    /// Timestamp option: 0=none, 1=date, 2=datetime, 3=nanoseconds
    #[arg(short = 't', long, default_value = "0")]
    pub timestamp: u8,

    /// Force overwrite without prompting
    #[arg(short = 'f', long)]
    pub force: bool,

    /// Input files or directories
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,
}

impl CliArgs {
    /// Validate arguments
    pub fn validate(&self) -> Result<(), String> {
        // Validate timestamp option
        if self.timestamp > 3 {
            return Err(format!("Invalid timestamp option: {}", self.timestamp));
        }

        // Validate compression command
        let valid_commands = ["gzip", "bzip2", "xz", "zip", "tar", "tgz", "tbz2", "txz"];
        if !valid_commands.contains(&self.command.as_str()) {
            return Err(format!("Invalid compression command: {}", self.command));
        }

        // Check that collect and collect_flat are not both specified
        if self.collect.is_some() && self.collect_flat.is_some() {
            return Err("Cannot specify both -a and -A".to_string());
        }

        Ok(())
    }
}
```

#### 3.5.2 Command Handlers (`src/cli/commands.rs`)

```rust
use std::path::PathBuf;

use crate::cli::args::CliArgs;
use crate::core::config::{CompressionConfig, CollectionConfig, CollectionMode, TimestampOption};
use crate::core::types::{CompressionFormat, CompoundFormat};
use crate::core::error::{JcError, JcResult};
use crate::operations::{compress, decompress, compound, collection};
use crate::utils::validation::{validate_input_files, validate_move_to};
use crate::utils::logger::{error, info};

/// Execute the appropriate command based on CLI arguments
pub fn execute(args: CliArgs) -> JcResult<()> {
    // Validate arguments
    args.validate()
        .map_err(|e| JcError::Other(e))?;

    // Build configuration
    let timestamp = TimestampOption::from_u8(args.timestamp)
        .ok_or(JcError::InvalidTimestampOption(args.timestamp))?;

    let config = CompressionConfig::new()
        .with_level(args.level)
        .with_timestamp(timestamp);

    let config = if let Some(ref move_to) = args.move_to {
        validate_move_to(move_to)?;
        config.with_move_to(move_to.clone())
    } else {
        config
    };

    // Validate input files
    let inputs = validate_input_files(args.inputs)?;
    let input_paths: Vec<PathBuf> = inputs.iter()
        .map(|f| f.real_path.clone())
        .collect();

    if args.decompress {
        // Decompression mode
        handle_decompress(input_paths, config)
    } else if args.collect.is_some() || args.collect_flat.is_some() {
        // Collection mode
        let mode = if args.collect.is_some() {
            CollectionMode::WithParent
        } else {
            CollectionMode::Flat
        };

        let package_name = args.collect
            .or(args.collect_flat)
            .unwrap();

        handle_collection(input_paths, &args.command, package_name, mode, config)
    } else {
        // Standard compression mode
        handle_compress(input_paths, &args.command, config)
    }
}

fn handle_decompress(inputs: Vec<PathBuf>, config: CompressionConfig) -> JcResult<()> {
    let results = decompress::decompress_files(inputs, config);

    // Check for errors
    let mut had_errors = false;
    for result in results {
        if let Err(e) = result {
            error!("Decompression failed: {}", e);
            had_errors = true;
        }
    }

    if had_errors {
        Err(JcError::Other("Some files failed to decompress".to_string()))
    } else {
        Ok(())
    }
}

fn handle_compress(
    inputs: Vec<PathBuf>,
    command: &str,
    config: CompressionConfig,
) -> JcResult<()> {
    // Determine if simple or compound format
    if let Some(compound) = CompoundFormat::from_str(command) {
        // Compound format (tgz, tbz2, txz)
        let results = compound::compress_compound_batch(inputs, compound, config);

        let mut had_errors = false;
        for result in results {
            if let Err(e) = result {
                error!("Compression failed: {}", e);
                had_errors = true;
            }
        }

        if had_errors {
            Err(JcError::Other("Some files failed to compress".to_string()))
        } else {
            Ok(())
        }
    } else {
        // Simple format (gzip, bzip2, xz, zip, tar)
        let format = CompressionFormat::from_extension(command)
            .ok_or_else(|| JcError::InvalidCommand(command.to_string()))?;

        let results = compress::compress_files(inputs, format, config);

        let mut had_errors = false;
        for result in results {
            if let Err(e) = result {
                error!("Compression failed: {}", e);
                had_errors = true;
            }
        }

        if had_errors {
            Err(JcError::Other("Some files failed to compress".to_string()))
        } else {
            Ok(())
        }
    }
}

fn handle_collection(
    inputs: Vec<PathBuf>,
    command: &str,
    package_name: String,
    mode: CollectionMode,
    config: CompressionConfig,
) -> JcResult<()> {
    let compound = CompoundFormat::from_str(command)
        .ok_or_else(|| JcError::InvalidCommand(command.to_string()))?;

    let collection_config = CollectionConfig {
        base: config,
        package_name,
        mode,
    };

    collection::collect_and_compress(inputs, compound, collection_config)?;

    Ok(())
}
```

#### 3.5.3 Main Entry Point (`src/main.rs`)

```rust
use clap::Parser;

mod cli;
mod core;
mod compressors;
mod operations;
mod utils;

use cli::args::CliArgs;
use cli::commands;
use utils::logger;

fn main() {
    // Initialize logging
    logger::init_logger();

    // Parse command-line arguments
    let args = CliArgs::parse();

    // Execute command
    match commands::execute(args) {
        Ok(()) => {
            std::process::exit(0);
        }
        Err(e) => {
            logger::error!("ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
```

---

## 4. Dependencies (`Cargo.toml`)

```toml
[package]
name = "jcz"
version = "0.1.0"
edition = "2021"
authors = ["JCZ Contributors"]
description = "Just Compress Zip - A unified compression utility"
license = "MIT"
repository = "https://github.com/saimizi/jcz"

[dependencies]
# Command-line argument parsing
clap = { version = "4.0", features = ["derive"] }

# Parallel processing
rayon = "1.7"

# Logging
log = "0.4"
env_logger = "0.10"

# Date/time for timestamps
chrono = "0.4"

[dev-dependencies]
# Testing utilities
tempfile = "3.8"
assert_cmd = "2.0"
predicates = "3.0"

[[bin]]
name = "jcz"
path = "src/main.rs"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## 5. Testing Strategy

### 5.1 Unit Tests

Each module should have associated unit tests:

```rust
// Example: src/utils/timestamp.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::TimestampOption;

    #[test]
    fn test_timestamp_none() {
        let ts = generate_timestamp(TimestampOption::None);
        assert_eq!(ts, "");
    }

    #[test]
    fn test_timestamp_date() {
        let ts = generate_timestamp(TimestampOption::Date);
        assert_eq!(ts.len(), 8); // YYYYMMDD
        assert!(ts.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_timestamp_datetime() {
        let ts = generate_timestamp(TimestampOption::DateTime);
        assert_eq!(ts.len(), 15); // YYYYMMDD_HHMMSS
        assert!(ts.contains('_'));
    }
}
```

### 5.2 Integration Tests

```rust
// tests/integration_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_compress_gzip() {
    let temp = TempDir::new().unwrap();
    let input_file = temp.path().join("test.txt");
    fs::write(&input_file, b"Hello, world!").unwrap();

    let mut cmd = Command::cargo_bin("jcz").unwrap();
    cmd.arg("-c").arg("gzip")
       .arg(&input_file)
       .current_dir(temp.path());

    cmd.assert().success();

    // Check output file exists
    let output_file = temp.path().join("test.txt.gz");
    assert!(output_file.exists());

    // Original file should still exist
    assert!(input_file.exists());
}

#[test]
fn test_decompress_gzip() {
    // Create a compressed file using gzip
    let temp = TempDir::new().unwrap();
    let input_file = temp.path().join("test.txt");
    fs::write(&input_file, b"Hello, world!").unwrap();

    // Compress it
    Command::new("gzip")
        .arg("-k")
        .arg(&input_file)
        .current_dir(temp.path())
        .assert()
        .success();

    // Remove original
    fs::remove_file(&input_file).unwrap();

    let compressed = temp.path().join("test.txt.gz");

    // Decompress with jcz
    let mut cmd = Command::cargo_bin("jcz").unwrap();
    cmd.arg("-d")
       .arg(&compressed)
       .current_dir(temp.path());

    cmd.assert().success();

    // Check decompressed file exists and has correct content
    assert!(input_file.exists());
    let content = fs::read_to_string(&input_file).unwrap();
    assert_eq!(content, "Hello, world!");
}
```

### 5.3 Test Coverage Goals

- **Unit Tests**: >80% code coverage
- **Integration Tests**: All major workflows (compress, decompress, collection)
- **Error Cases**: Validate error handling for invalid inputs
- **Edge Cases**: Empty files, large files, special characters in filenames

---

## 6. Build and Development

### 6.1 Development Workflow

```bash
# Build debug version
cargo build

# Run with debug logging
JCZDBG=debug cargo run -- -c gzip test.txt

# Run tests
cargo test

# Run specific test
cargo test test_compress_gzip

# Check code
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Build release version
cargo build --release
```

### 6.2 CI/CD Pipeline (GitHub Actions Example)

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Install compression tools
        run: |
          sudo apt-get update
          sudo apt-get install -y gzip bzip2 xz-utils zip unzip tar

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Lint with clippy
        run: cargo clippy -- -D warnings

      - name: Run tests
        run: cargo test --verbose

      - name: Build release
        run: cargo build --release --verbose
```

---

## 7. Performance Considerations

### 7.1 Concurrency

- **Rayon**: Work-stealing parallelism for batch operations
- **Thread Pool**: Rayon manages thread pool automatically
- **Send/Sync**: All shared types implement Send + Sync for safety

### 7.2 Memory Efficiency

- **Streaming I/O**: Use pipes and buffered readers/writers
- **Avoid Loading Entire Files**: Stream data through compression tools
- **Temporary File Cleanup**: RAII guards ensure cleanup

### 7.3 I/O Optimization

- **BufWriter**: Buffer writes to reduce syscalls
- **BufReader**: Buffer reads for efficiency
- **File Size Checks**: Validate disk space before operations (future enhancement)

---

## 8. Error Handling Philosophy

### 8.1 Recoverable vs. Unrecoverable

- **Recoverable**: Use `Result<T, JcError>` for expected failures
- **Unrecoverable**: Use `panic!` only for programmer errors (invariant violations)

### 8.2 Error Propagation

- **? operator**: Propagate errors up the call stack
- **Context**: Wrap errors with context using custom error types
- **Logging**: Log errors at appropriate levels before returning

### 8.3 User-Friendly Messages

- **Display trait**: Provide clear, actionable error messages
- **Error context**: Include relevant paths and details
- **Suggestions**: Hint at solutions where possible

---

## 9. Future Enhancements

### 9.1 Planned Features

1. **Pure Rust Compression**: Implement compression algorithms natively
   - Use `flate2` crate for GZIP
   - Use `bzip2` crate for BZIP2
   - Use `xz2` or `lzma-rs` for XZ
   - Implement TAR format reading/writing

2. **Progress Bars**: Show progress for large files
   - Use `indicatif` crate

3. **Parallel Compression**: Compress single file using multiple threads
   - Split large files into chunks
   - Compress chunks in parallel

4. **Integrity Checking**: Verify compressed files
   - Compute checksums
   - Verify after compression

5. **Configuration File**: Support `.jcrc` for defaults

### 9.2 Extensibility Points

- **Compressor Trait**: Add new compression formats
- **Plugin System**: Load compressors dynamically
- **Custom Backends**: Alternative implementations (pure Rust vs. external tools)

---

## 10. Security Considerations

### 10.1 Path Traversal Prevention

- Validate all input paths
- Canonicalize paths before operations
- Prevent writing outside designated directories

### 10.2 Symlink Handling

- Resolve symlinks to prevent confusion attacks
- Validate resolved paths

### 10.3 Resource Limits

- Implement timeouts for external processes (future)
- Limit temporary directory size (future)
- Validate file sizes before operations (future)

---

## 11. Appendices

### Appendix A: Rust Ecosystem Alignment

This design aligns with Rust best practices:

- **Cargo**: Standard build tool and package manager
- **Clap**: Industry-standard CLI parsing
- **Rayon**: De facto standard for data parallelism
- **Log/env_logger**: Standard logging infrastructure
- **Error Handling**: Idiomatic `Result` types

### Appendix B: Performance Benchmarks

To be added after implementation:

- Compression speed vs. external tools
- Memory usage profiling
- Parallel vs. sequential performance

### Appendix C: Migration from Go Version

Key differences:

1. **Memory Management**: Rust ownership vs. Go GC
2. **Concurrency**: Rayon vs. goroutines
3. **Error Handling**: `Result` vs. multi-return values
4. **Type System**: Stronger compile-time guarantees

### Appendix D: API Stability

- **Public API**: Defined in `lib.rs`
- **Semver**: Follow semantic versioning
- **Breaking Changes**: Only in major versions
- **Deprecation**: Warn for at least one minor version before removal

---

**End of Software Design Document**
