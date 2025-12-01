# Product Overview

JCZ (Just Compress Zip) is a unified command-line compression utility written in Rust that provides a consistent interface for multiple compression formats.

## Core Purpose

Simplify file and directory compression/decompression by providing a single command that wraps system compression tools (gzip, bzip2, xz, zip, tar) with a consistent, user-friendly interface.

## Key Features

- **Multi-format support**: GZIP, BZIP2, XZ, ZIP, TAR, and compound formats (TGZ, TBZ2, TXZ)
- **File encryption**: Password-based (AES-256-GCM) and RSA public-key encryption
- **Parallel processing**: Concurrent compression/decompression using Rayon
- **Timestamp options**: Add date/time stamps to output filenames
- **File collection**: Combine multiple files into single archives
- **Automatic format detection**: Smart decompression of compound formats
- **Original file preservation**: Always keeps original files intact

## Target Users

- System administrators needing reliable, scriptable compression tools
- Developers requiring automation-friendly utilities
- Power users wanting advanced features like batch processing
- Regular users seeking simple compression operations

## Design Philosophy

- **Consistent interface**: Single command for all compression formats
- **Safety first**: Preserve original files, validate inputs
- **Performance**: Leverage Rust's zero-cost abstractions and parallel processing
- **User-friendly**: Clear error messages, sensible defaults
