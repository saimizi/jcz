# JCZ - Just Compress Zip (Rust Implementation)

A unified command-line compression utility written in Rust, providing a consistent interface for multiple compression formats.

## Features

- **Multi-Format Support**: GZIP, BZIP2, XZ, ZIP, TAR, and compound formats (TGZ, TBZ2, TXZ)
- **File Encryption**: Password-based and RSA public-key encryption for compressed files
- **Parallel Processing**: Concurrent compression/decompression of multiple files using Rayon
- **Timestamp Options**: Add timestamps to output filenames (date, datetime, or nanoseconds)
- **File Collection**: Combine multiple files into single archives
- **Automatic Format Detection**: Smart decompression of compound formats
- **Original File Preservation**: Always keeps original files intact
- **Configurable Compression Levels**: 1-9 for supported algorithms

## Installation

### From Source

```bash
cargo build --release
sudo cp target/release/jcz /usr/local/bin/
```

## Usage

### Basic Compression

```bash
# Compress with GZIP
jcz -c gzip file.txt

# Compress with BZIP2 at level 9
jcz -c bzip2 -l 9 file.txt

# Compress with ZIP
jcz -c zip file.txt

# Create TAR archive
jcz -c tar directory/
```

### Compound Formats

```bash
# Create .tar.gz
jcz -c tgz directory/

# Create .tar.bz2
jcz -c tbz2 file1.txt file2.txt

# Create .tar.xz
jcz -c txz myfiles/
```

### Decompression

```bash
# Decompress any supported format
jcz -d archive.tar.gz

# Decompress multiple files
jcz -d file1.gz file2.bz2 file3.xz
```

### Encryption

```bash
# Encrypt with password
jcz -c gzip -e file.txt
# Output: file.txt.gz.jcze (encrypted)

# Encrypt with RSA public key
jcz -c gzip --encrypt-key public.pem file.txt
# Output: file.txt.gz.jcze (encrypted)

# Decrypt password-encrypted file (will prompt for password)
jcz -d file.txt.gz.jcze

# Decrypt RSA-encrypted file with private key
jcz -d --decrypt-key private.pem file.txt.gz.jcze

# Encrypt compound format
jcz -c tgz -e directory/
# Output: directory.tar.gz.jcze
```

### Advanced Features

```bash
# Add timestamp to output filename
jcz -c gzip -t 2 file.txt
# Output: file.txt_20251101_121019.gz

# Move compressed files to directory
jcz -c gzip -C /backups/ *.txt

# Collect files into archive with parent directory
jcz -c tgz -a myarchive file1.txt file2.txt dir/

# Collect files without parent directory wrapper
jcz -c tgz -A myarchive file1.txt file2.txt

# Combine encryption with other options
jcz -c gzip -e -t 2 -C /secure/ file.txt
# Output: /secure/file.txt_20251201_143022.gz.jcze
```

### Options

```
-d, --decompress                   Decompress mode
-c, --command <COMMAND>            Compression command [default: tgz]
-l, --level <LEVEL>                Compression level (1-9) [default: 6]
-C, --move-to <MOVE_TO>            Move compressed file to specified directory
-a, --collect <COLLECT>            Collect files into archive (with parent directory)
-A, --collect-flat <COLLECT_FLAT>  Collect files into archive (flat, without parent directory)
-t, --timestamp <TIMESTAMP>        Timestamp option: 0=none, 1=date, 2=datetime, 3=nanoseconds [default: 0]
-e, --encrypt-password             Enable password-based encryption
    --encrypt-key <FILE>           RSA public key file for encryption
    --decrypt-key <FILE>           RSA private key file for decryption
-f, --force                        Force overwrite without prompting
-h, --help                         Print help
-V, --version                      Print version
```

### Supported Commands

- `gzip` - GZIP compression (.gz)
- `bzip2` - BZIP2 compression (.bz2)
- `xz` - XZ compression (.xz)
- `zip` - ZIP compression (.zip)
- `tar` - TAR archive (.tar)
- `tgz` - TAR + GZIP (.tar.gz)
- `tbz2` - TAR + BZIP2 (.tar.bz2)
- `txz` - TAR + XZ (.tar.xz)

## Environment Variables

- `JCDBG` - Control logging verbosity
  - `error` - Show only errors
  - `warn` - Show warnings and errors
  - `info` - Show info, warnings, and errors (default)
  - `debug` - Show all log messages including debug

```bash
JCDBG=debug jcz -c gzip file.txt
```

## Architecture

The implementation follows a modular design:

- **Core Module**: Trait definitions, error types, configuration structures
- **Compressor Modules**: Individual implementations for GZIP, BZIP2, XZ, ZIP, TAR
- **Crypto Module**: Encryption/decryption with password and RSA support
- **Operations Module**: High-level operations (compress, decompress, encrypt, decrypt, compound, collection)
- **Utils Module**: File system utilities, logging, validation, timestamp generation
- **CLI Module**: Command-line argument parsing and command execution

## Encryption Details

### Password-Based Encryption
- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key Derivation**: Argon2id with secure parameters (64MB memory, 3 iterations)
- **Security**: Cryptographically secure random salt and nonce generation
- **File Extension**: `.jcze` (JCZ Encrypted)

### RSA Encryption
- **Algorithm**: RSA with OAEP-SHA256 padding
- **Key Size**: Minimum 2048 bits (4096 bits recommended)
- **Hybrid Approach**: RSA encrypts a random AES-256 symmetric key, which encrypts the data
- **Key Format**: PEM-encoded public/private keys
- **Security**: Public key encrypts, private key decrypts (standard RSA confidentiality)

## Design Highlights

- **Trait-based polymorphism**: All compressors implement the `Compressor` trait
- **Error handling**: Comprehensive error types using Rust's `Result<T, E>` pattern
- **Concurrency**: Safe parallel processing with Rayon's work-stealing algorithm
- **RAII**: Automatic cleanup of temporary files using guard patterns
- **Zero-cost abstractions**: Generic and trait-based design with no runtime overhead

## Dependencies

- `clap` - Command-line argument parsing
- `rayon` - Data parallelism
- `log` / `env_logger` - Logging infrastructure
- `chrono` - Timestamp generation
- `ring` - AES-256-GCM encryption
- `rsa` - RSA public-key cryptography
- `argon2` - Password-based key derivation
- `rpassword` - Secure password input

## System Requirements

- Rust 2021 edition or later
- System utilities: `gzip`, `bzip2`, `xz`, `zip`, `unzip`, `tar`, `mv`, `cp`, `readlink`

## Documentation

- [Software Requirements Specification](docs/jcz_srs.md)
- [Software Design Document](docs/jcz_sdd.md)

## License

MIT

