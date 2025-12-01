# Technology Stack

## Language & Edition

- **Rust 2021 edition**
- Minimum toolchain: stable Rust compiler

## Build System

**Cargo** - Standard Rust package manager and build tool

### Common Commands

```bash
# Build debug version
cargo build

# Build optimized release version
cargo build --release

# Run the application
cargo run -- [args]

# Run tests
cargo test

# Run specific test
cargo test test_name

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Install locally
cargo build --release
sudo cp target/release/jcz /usr/local/bin/
```

## Core Dependencies

### CLI & Argument Parsing
- **clap** (v4.0) with derive features - Command-line argument parsing

### Concurrency
- **rayon** (v1.7) - Data parallelism and work-stealing for concurrent file processing

### Logging
- **log** (v0.4) - Logging facade
- **env_logger** (v0.10) - Logger implementation controlled by `JCDBG` environment variable

### Date/Time
- **chrono** (v0.4) - Timestamp generation for filename suffixes

### File System
- **tempfile** (v3.8) - Temporary directory management with automatic cleanup

### Cryptography
- **ring** (v0.17) - AES-256-GCM encryption
- **rsa** (v0.9) - RSA public-key cryptography
- **argon2** (v0.5) - Password-based key derivation
- **pem** (v3.0) - PEM file parsing
- **sha2** (v0.10) - SHA-256 hashing
- **rand** (v0.8) - Cryptographically secure random number generation
- **zeroize** (v1.7) - Secure memory zeroing for sensitive data
- **rpassword** (v7.3) - Secure password input without echo

### Testing
- **assert_cmd** (v2.0) - CLI testing utilities
- **predicates** (v3.0) - Assertion predicates for tests
- **proptest** (v1.4) - Property-based testing

## External System Dependencies

The application wraps these system utilities (must be in PATH):
- `gzip` - GZIP compression/decompression
- `bzip2` - BZIP2 compression/decompression
- `xz` - XZ compression/decompression
- `zip` - ZIP compression
- `unzip` - ZIP decompression
- `tar` - TAR archive creation/extraction
- `mv` - File moving
- `cp` - File copying
- `readlink` - Symbolic link resolution

## Release Profile

Optimized for performance:
```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
```

## Environment Variables

- `JCDBG` - Control logging verbosity
  - `error` - Only errors
  - `warn` - Warnings and errors
  - `info` - Info, warnings, errors (default)
  - `debug` - All messages including debug
