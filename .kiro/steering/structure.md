# Project Structure

## Module Organization

```
src/
├── main.rs              # Entry point, CLI initialization
├── lib.rs               # Library root, public API exports
├── cli/                 # Command-line interface
│   ├── mod.rs           # CLI module root
│   ├── args.rs          # Argument parsing with clap
│   └── commands.rs      # Command execution logic
├── core/                # Core abstractions and types
│   ├── mod.rs           # Core module root
│   ├── compressor.rs    # Compressor trait definitions
│   ├── config.rs        # Configuration structures
│   ├── error.rs         # Error types (JcError, JcResult)
│   └── types.rs         # Common type definitions
├── compressors/         # Compression implementations
│   ├── mod.rs           # Compressor module root
│   ├── gzip.rs          # GZIP implementation
│   ├── bzip2.rs         # BZIP2 implementation
│   ├── xz.rs            # XZ implementation
│   ├── zip.rs           # ZIP implementation
│   └── tar.rs           # TAR implementation
├── crypto/              # Encryption/decryption
│   ├── mod.rs           # Crypto module root
│   ├── container.rs     # Encrypted container format
│   ├── keys.rs          # Key management
│   ├── password.rs      # Password-based encryption
│   └── rsa.rs           # RSA public-key encryption
├── operations/          # High-level operations
│   ├── mod.rs           # Operations module root
│   ├── compress.rs      # Compression operations
│   ├── decompress.rs    # Decompression operations
│   ├── encrypt.rs       # Encryption operations
│   ├── decrypt.rs       # Decryption operations
│   ├── collection.rs    # Multi-file collection
│   └── compound.rs      # Compound format handling
└── utils/               # Utility functions
    ├── mod.rs           # Utils module root
    ├── fs.rs            # File system utilities
    ├── logger.rs        # Logging initialization
    ├── prompt.rs        # User prompts
    ├── timestamp.rs     # Timestamp generation
    └── validation.rs    # Input validation
```

## Architecture Layers

### 1. CLI Layer (`cli/`)
- Parses command-line arguments using clap
- Dispatches to appropriate operations
- Handles user interaction and output

### 2. Operations Layer (`operations/`)
- High-level business logic
- Orchestrates compression/decompression workflows
- Handles batch processing with Rayon
- Manages temporary directories and cleanup

### 3. Core Abstraction Layer (`core/`)
- **Compressor trait**: Common interface for all compressors
- **MultiFileCompressor trait**: Extended interface for archive formats
- **Error types**: Comprehensive error handling with JcError enum
- **Configuration**: Shared config structures (CompressionConfig, CollectionConfig)

### 4. Implementation Layer (`compressors/`, `crypto/`)
- Concrete implementations of Compressor trait
- Each compressor in its own module
- Wraps system utilities via std::process::Command
- Crypto operations for encryption/decryption

### 5. Utility Layer (`utils/`)
- Cross-cutting concerns
- File system operations
- Logging setup
- Validation helpers

## Key Design Patterns

### Trait-Based Polymorphism
All compressors implement the `Compressor` trait, enabling uniform handling:
```rust
pub trait Compressor: Send + Sync {
    fn name(&self) -> &'static str;
    fn extension(&self) -> &'static str;
    fn compress(&self, input: &Path, config: &CompressionConfig) -> JcResult<PathBuf>;
    fn decompress(&self, input: &Path, config: &CompressionConfig) -> JcResult<PathBuf>;
    fn supports_levels(&self) -> bool;
    fn validate_level(&self, level: u8) -> bool;
    fn default_level(&self) -> u8;
}
```

### Error Handling
- Custom error type `JcError` with specific variants
- Type alias `JcResult<T> = Result<T, JcError>`
- All operations return `JcResult` for consistent error propagation
- Automatic conversion from `io::Error` via `From` trait

### Configuration Pattern
- Immutable configuration structures
- Builder pattern with `with_*` methods for fluent API
- Separation: base config vs. specialized configs (CollectionConfig)

### RAII for Resource Management
- Temporary directories cleaned up automatically via Drop
- File handles closed automatically
- Guard patterns for cleanup on error

### Concurrent Processing
- Rayon for parallel file processing
- Thread-safe compressor implementations (Send + Sync)
- Independent file operations processed concurrently

## Module Dependencies

- `cli` → `operations` → `compressors` + `crypto` → `core` + `utils`
- `core` defines interfaces, `compressors` implements them
- `utils` is a leaf module used by all layers
- No circular dependencies

## Testing Structure

```
tests/
├── common/              # Shared test utilities
│   └── mod.rs
├── test_compression.rs  # Compression tests
├── test_decompression.rs # Decompression tests
├── test_compound.rs     # Compound format tests
├── test_errors.rs       # Error handling tests
├── test_options.rs      # CLI options tests
└── test_*.rs            # Format-specific tests
```

## Documentation

```
docs/
├── jcz_srs.md          # Software Requirements Specification
└── jcz_sdd.md          # Software Design Document
```
