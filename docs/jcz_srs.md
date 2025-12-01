# Software Requirements Specification (SRS)
## JCZ - Just Compress Zip Utility

**Version:** 1.2
**Date:** 2025-12-01
**Document Status:** Final

---

## 1. Introduction

### 1.1 Purpose
This document specifies the functional and non-functional requirements for JCZ (Just Compress Zip), a command-line compression utility that provides a unified interface for multiple compression formats.

### 1.2 Scope
JCZ is a command-line tool that simplifies file and directory compression/decompression operations. It supports multiple compression algorithms (GZIP, BZIP2, XZ, ZIP) and archive formats (TAR), including compound formats (TAR+GZIP, TAR+BZIP2, TAR+XZ).

### 1.3 Intended Audience
- Software developers implementing the tool
- Quality assurance engineers
- End users seeking to understand tool capabilities
- System administrators

### 1.4 Definitions and Acronyms
- **JCZ**: Just Compress Zip
- **CLI**: Command-Line Interface
- **GZIP**: GNU Zip compression algorithm
- **BZIP2**: Burrows-Wheeler compression algorithm
- **XZ**: LZMA/LZMA2 compression algorithm
- **ZIP**: ZIP compression and archive format
- **TAR**: Tape Archive format
- **TGZ**: TAR + GZIP compound format (.tar.gz)
- **TBZ2**: TAR + BZIP2 compound format (.tar.bz2)
- **TXZ**: TAR + XZ compound format (.tar.xz)
- **AES-GCM**: Advanced Encryption Standard in Galois/Counter Mode
- **RSA**: Rivest-Shamir-Adleman public-key cryptosystem
- **Argon2id**: Memory-hard password hashing function
- **JCZE**: JCZ Encrypted file extension (.jcze)

---

## 2. Overall Description

### 2.1 Product Perspective
JCZ is a standalone command-line utility that wraps system compression tools (gzip, bzip2, xz, zip, unzip, tar) to provide a consistent, simplified interface. It acts as a compression abstraction layer, allowing users to compress/decompress files without memorizing different command syntaxes for each compression tool.

### 2.2 Product Features
1. **Multi-Format Compression**: Support for GZIP, BZIP2, XZ, ZIP, TAR, TGZ, TBZ2, TXZ
2. **Multi-Format Decompression**: Automatic format detection and sequential decompression
3. **File Encryption**: Password-based and RSA public-key encryption for compressed files
4. **Secure Decryption**: Automatic encrypted file detection and decryption
5. **Isolated Decompression**: Temporary directory isolation prevents file conflicts
6. **Force Overwrite**: Skip interactive prompts with --force/-f flag
7. **Auto-Create Directories**: Automatically create destination directories with -C flag
8. **Multi-File Archive Support**: Correctly handle TAR archives with multiple files
9. **Configurable Compression Levels**: User-selectable compression ratios (1-9)
8. **Timestamp Appending**: Optional timestamp suffixes for output files
9. **File Collection**: Combine multiple files into single archive
10. **Output Relocation**: Move compressed/decompressed files to specified directories
11. **Concurrent Processing**: Parallel compression/decompression of multiple files
12. **Symbolic Link Resolution**: Process actual files instead of symbolic links
13. **Logging System**: Configurable logging with multiple severity levels

### 2.3 User Classes and Characteristics
- **System Administrators**: Need reliable, scriptable compression tools
- **Developers**: Require automation-friendly compression utilities
- **Power Users**: Want advanced features like timestamp naming and batch processing
- **Regular Users**: Seek simple compression/decompression operations

### 2.4 Operating Environment
- **Operating Systems**: Linux, Unix-like systems
- **Dependencies**: System utilities (gzip, bzip2, xz, zip, unzip, tar, mv, cp, readlink)
- **Interface**: Command-line terminal
- **File Systems**: Any POSIX-compatible filesystem

### 2.5 Design and Implementation Constraints
1. Must use external system compression tools (gzip, bzip2, xz, zip, unzip, tar)
2. Must preserve original files during compression operations
3. Must handle file paths up to system limits
4. Must operate within terminal environment constraints
5. Must maintain cross-platform compatibility with Unix-like systems

---

## 3. System Features and Requirements

### 3.1 Functional Requirements

#### 3.1.1 Compression Operations

##### FR-COMP-001: Single File Compression
**Description**: The system shall compress individual files using GZIP, BZIP2, XZ, or ZIP algorithms.

**Inputs**:
- Input file path (file or directory for TAR/ZIP)
- Compression algorithm (gzip, bzip2, xz, zip, tar)
- Optional: Compression level (1-9)
- Optional: Timestamp option (0-3)
- Optional: Destination directory

**Processing**:
1. Validate input file exists and is accessible
2. Validate compression algorithm is supported
3. Validate compression level is within valid range
4. Invoke appropriate compression algorithm
5. Preserve original file (keep original)
6. Generate output filename based on algorithm and timestamp option
7. Optionally move output to destination directory

**Outputs**:
- Compressed file with appropriate extension (.gz, .bz2, .xz, .tar)
- Success/error status
- Log messages

**Error Conditions**:
- Input file not found
- Input file not readable
- Insufficient disk space
- Permission denied on destination directory
- Invalid compression level
- Compression tool not available

##### FR-COMP-002: Multi-File Archive Compression
**Description**: The system shall collect multiple files into a single archive and optionally compress it.

**Inputs**:
- Multiple input file/directory paths
- Archive name
- Compression algorithm (tar, tgz, tbz2, txz)
- Optional: Compression level (1-9 for compressed archives)
- Optional: Timestamp option (0-3)
- Optional: Destination directory
- Optional: Parent directory inclusion flag

**Processing**:
1. Validate all input files exist
2. Check for duplicate filenames (basename collision detection)
3. Create temporary directory
4. Copy all input files to temporary directory
5. Create TAR archive from collected files
6. Optionally apply secondary compression (GZIP/BZIP2/XZ)
7. Move final archive to destination or current directory
8. Clean up temporary files

**Outputs**:
- Archive file (.tar, .tar.gz, .tar.bz2, .tar.xz)
- Success/error status
- Log messages

**Error Conditions**:
- Input files not found
- Duplicate filenames when -a/-A flag used
- Archive name already exists
- Insufficient disk space
- Permission errors

##### FR-COMP-003: Two-Step Compound Compression
**Description**: The system shall perform sequential TAR+compression operations for compound formats.

**Inputs**:
- Input file(s) or directory(ies)
- Compound format (tgz, tbz2, txz)
- Optional: Compression level (1-9)
- Optional: Timestamp option (0-3)
- Optional: Destination directory

**Processing**:
1. Create TAR archive from input
2. Apply secondary compression to TAR file
3. Remove intermediate TAR file
4. Optionally move final compressed archive to destination

**Outputs**:
- Compound compressed file (.tar.gz, .tar.bz2, .tar.xz)
- Success/error status

**Error Conditions**:
- TAR creation failure
- Compression failure
- Same-named output files without unique timestamps

#### 3.1.2 Decompression Operations

##### FR-DECOMP-001: Single Format Decompression
**Description**: The system shall decompress files in GZIP, BZIP2, XZ, or TAR formats.

**Inputs**:
- Compressed file path
- Optional: Destination directory

**Processing**:
1. Detect compression format from file extension
2. Validate file has supported extension
3. Invoke appropriate decompression algorithm
4. Preserve original compressed file
5. Optionally move decompressed output to destination

**Outputs**:
- Decompressed file
- Success/error status

**Error Conditions**:
- Invalid file extension
- Corrupted compressed file
- Decompression tool failure
- Destination directory not found

##### FR-DECOMP-002: Compound Format Decompression
**Description**: The system shall automatically detect and sequentially decompress compound formats.

**Inputs**:
- Compound compressed file (.tar.gz, .tar.bz2, .tar.xz)
- Optional: Destination directory

**Processing**:
1. Iteratively detect compression layers from file extension
2. Decompress outer layer (e.g., GZIP)
3. Remove intermediate file
4. Decompress inner layer (e.g., TAR)
5. Continue until no compression detected
6. Optionally move final output to destination

**Outputs**:
- Fully decompressed file(s) or directory
- Success/error status

**Error Conditions**:
- Unsupported compression layer
- Corruption in any layer
- Intermediate decompression failure

##### FR-DECOMP-003: Batch Decompression
**Description**: The system shall decompress multiple files concurrently.

**Inputs**:
- Multiple compressed file paths
- Optional: Destination directory

**Processing**:
1. Validate all input files
2. Remove duplicate file references
3. Spawn concurrent decompression operations
4. Wait for all operations to complete
5. Report individual file success/failure

**Outputs**:
- Multiple decompressed files
- Aggregate success/error status

**Error Conditions**:
- Any individual file decompression failure

##### FR-DECOMP-004: Temporary Directory Isolation
**Description**: The system shall use isolated temporary directories for all decompression operations to prevent conflicts with existing files.

**Inputs**:
- Compressed file path
- Optional: Destination directory

**Processing**:
1. Create unique temporary directory in /tmp
2. Copy input file to temporary directory
3. Perform all decompression operations in temporary directory
4. For compound formats, perform iterative decompression in same temporary directory
5. Copy final decompressed output to destination
6. Automatically clean up temporary directory on completion or error

**Outputs**:
- Decompressed file(s) in final destination
- No residual temporary files

**Benefits**:
- Prevents conflicts with existing intermediate files (e.g., .tar from .tar.gz)
- Isolated working environment for each decompression operation
- Automatic cleanup even on interruption (RAII pattern)
- Multiple concurrent decompressions don't interfere with each other

**Error Conditions**:
- Failed to create temporary directory
- Insufficient /tmp space

##### FR-DECOMP-005: Force Overwrite Option
**Description**: The system shall provide a force flag to skip overwrite prompts during decompression.

**Inputs**:
- Compressed file path
- Optional: --force/-f flag
- Optional: Destination directory

**Processing**:
1. When force flag is NOT specified:
   - Check if destination file/directory already exists
   - For multi-file archives: prompt for each individual file
   - For single files: prompt once before extraction
   - Allow user to skip individual files (respond 'n')
   - Allow user to overwrite files (respond 'y')
2. When force flag IS specified:
   - Skip all overwrite prompts
   - Automatically overwrite existing files

**Outputs**:
- Decompressed files (existing files overwritten or skipped based on user choice)
- Log messages indicating skipped files

**User Interaction**:
- Prompt format: "File 'path' already exists. Overwrite? (y/n): "
- Case-insensitive responses: y/yes/n/no

**Error Conditions**:
- User aborts decompression (responds 'n' to overwrite prompt)

##### FR-DECOMP-006: Automatic Destination Directory Creation
**Description**: The system shall automatically create destination directories specified with -C flag if they don't exist.

**Inputs**:
- Compressed file path
- Destination directory path (via -C flag)

**Processing**:
1. Check if destination directory exists
2. If not exists: create directory and all parent directories (like mkdir -p)
3. If exists: verify it is a directory and writable
4. Proceed with decompression to the directory

**Outputs**:
- Created directory structure
- Decompressed files in destination

**Benefits**:
- User-friendly: no need to pre-create directories
- Consistent with standard tools (tar -C)
- Supports nested paths

**Error Conditions**:
- Failed to create directory (permission denied)
- Path exists but is not a directory
- Directory not writable

##### FR-DECOMP-007: Multi-File Archive Extraction
**Description**: The system shall correctly handle TAR archives containing multiple files or directories.

**Inputs**:
- TAR or compound TAR archive (.tar, .tar.gz, .tar.bz2, .tar.xz)
- Optional: Destination directory via -C flag

**Processing**:
1. Extract all files from archive to temporary directory
2. Detect extraction result:
   - Single file: copy file to destination
   - Single directory: copy directory to destination
   - Multiple files: copy all files directly to destination
3. When using -C flag with multiple files:
   - Extract files directly to specified directory
   - Do NOT create subdirectory based on archive name
4. When NOT using -C flag with multiple files:
   - Create subdirectory based on archive name
   - Extract files into that subdirectory

**Outputs**:
- All extracted files in correct locations

**Examples**:
- `jcz -d multi.tar.gz -C output/` → files in `output/file1.txt`, `output/file2.txt`
- `jcz -d multi.tar.gz` → files in `multi/file1.txt`, `multi/file2.txt`
- `jcz -d single.tar.gz -C output/` → file in `output/single.txt`

**Error Conditions**:
- Corrupted TAR archive
- Extraction failure

#### 3.1.3 Configuration and Options

##### FR-CONFIG-001: Compression Level Configuration
**Description**: The system shall allow users to specify compression levels.

**Requirements**:
- GZIP: Levels 1-9 (1=fastest, 9=best compression)
- BZIP2: Levels 1-9 (1=fastest, 9=best compression)
- XZ: Levels 0-9 (0=fastest, 9=best compression, default=6)
- TAR: No compression level (setting level should succeed as no-op)
- Default compression level: 6 (command-line default)
- Invalid levels shall be rejected with error message

##### FR-CONFIG-002: Timestamp Options
**Description**: The system shall support appending timestamps to output filenames.

**Options**:
- Option 0: No timestamp
- Option 1: Date only (YYYYMMDD format)
- Option 2: Date and time (YYYYMMDD_HHMMSS format)
- Option 3: Nanoseconds (nanosecond component only)

**Behavior**:
- Timestamp inserted between base filename and extension
- Format: `filename_timestamp.extension`
- Example: `file.txt` → `file.txt_20251101_143522.gz`

##### FR-CONFIG-003: Output Directory Configuration
**Description**: The system shall support moving compressed/decompressed files to specified directories.

**Requirements**:
- Validate destination directory exists
- Validate destination is a directory (not a file)
- Validate write permissions on destination
- Preserve basename of file when moving
- Handle trailing slash in directory path

##### FR-CONFIG-004: File Collection Modes
**Description**: The system shall support two collection modes for archiving.

**Modes**:
- **Standard Collection (-a)**: Include parent directory in archive
- **Flat Collection (-A)**: Archive files without parent directory wrapper

**Requirements**:
- Prohibit duplicate basenames in collection mode
- Create temporary staging directory for collection
- Clean up temporary files after completion

#### 3.1.4 Input Validation

##### FR-VALID-001: Input File Validation
**Description**: The system shall validate input files before processing.

**Validation Checks**:
1. File existence verification
2. File readability verification
3. Symbolic link resolution (resolve to real file)
4. Duplicate file detection (by absolute path)
5. Duplicate basename detection (for collection operations)
6. Directory vs. file detection

**Requirements**:
- Symbolic links shall be resolved to real file paths
- Duplicate paths shall be deduplicated
- Files with same basename from different directories shall be detected

##### FR-VALID-002: Compression Command Validation
**Description**: The system shall validate compression commands.

**Valid Commands**:
- `gzip`
- `bzip2`
- `xz`
- `zip`
- `tar`
- `tgz`
- `tbz2`
- `txz`

**Requirements**:
- Invalid commands shall be rejected
- Error message shall list valid commands

##### FR-VALID-003: File Extension Validation for Decompression
**Description**: The system shall validate file extensions for decompression.

**Valid Extensions**:
- `.gz`
- `.bz2`
- `.xz`
- `.zip`
- `.tar`
- `.tar.gz`
- `.tar.bz2`
- `.tar.xz`

**Requirements**:
- Invalid extensions shall be rejected
- Error shall specify invalid suffix

#### 3.1.5 Logging System

##### FR-LOG-001: Multi-Level Logging
**Description**: The system shall support configurable logging levels.

**Log Levels**:
1. **ERROR**: Critical errors (always enabled)
2. **WARN**: Warnings (enabled by default)
3. **INFO**: Informational messages (enabled by default)
4. **DEBUG**: Debug messages (enabled only when `JCZDBG=debug`)

**Requirements**:
- ERROR logs shall include file location (short filename)
- DEBUG logs shall include file location (short filename)
- Log output shall go to STDERR
- Disabled log levels shall output to /dev/null
- Environment variable `JCZDBG` shall control logging verbosity

**JCZDBG Environment Variable Values**:
- `error`: Only error logs
- `warn`: Error and warning logs
- `info`: Error, warning, and info logs (default behavior)
- `debug`: All log levels including debug
- unset/other: Default to info level

##### FR-LOG-002: Logging Content
**Description**: The system shall log relevant operational information.

**Log Messages**:
- Compression/decompression operations
- File movements
- Configuration settings
- Temporary file creation/removal
- Symbolic link resolution
- Error conditions
- Warning conditions (e.g., failed compression level setting)

---

### 3.2 External Interface Requirements

#### 3.2.1 User Interface

##### UI-001: Command-Line Interface
**Description**: The system shall provide a command-line interface.

**Command Syntax**:
```
jcz [Options] <File|Dir> [File|Dir]...
```

**Options**:
- `-d`: Decompress mode (default: compress mode)
- `-c <cmd>`: Compression command (default: tgz)
- `-l <level>`: Compression level 1-9 (default: 6)
- `-C <dir>`: Move output to specified directory (default: current directory)
- `-a <name>`: Collect files into archive with parent directory
- `-A <name>`: Collect files into archive without parent directory
- `-t <option>`: Timestamp option 0-3 (default: 0)

**Help/Usage**:
- Tool shall display usage information
- Usage shall list available options
- Usage shall list available compression commands with extensions

##### UI-002: Exit Codes
**Description**: The system shall return appropriate exit codes.

**Exit Codes**:
- `0`: Success
- `1`: Error (file not found, invalid arguments, compression failure, etc.)

#### 3.2.2 System Interface

##### SI-001: External Tool Dependencies
**Description**: The system shall interface with external compression tools.

**Required External Tools**:
- `gzip`: For GZIP compression/decompression
- `bzip2`: For BZIP2 compression/decompression
- `xz`: For XZ compression/decompression
- `zip`: For ZIP compression
- `unzip`: For ZIP decompression
- `tar`: For TAR archive creation/extraction
- `mv`: For moving files
- `cp`: For copying files
- `readlink`: For resolving symbolic links

**Interface Requirements**:
- Tools shall be invoked via system process execution
- STDOUT shall be captured for compressed data streaming
- STDERR shall be captured for error messages
- Tool exit codes shall be checked for success/failure

##### SI-002: File System Interface
**Description**: The system shall interact with the file system.

**Operations**:
- File existence checking
- File metadata retrieval (size, type, permissions)
- Symbolic link detection and resolution
- Directory creation (temporary directories)
- File creation (compressed output)
- File deletion (temporary/intermediate files)
- File movement (to destination directories)
- File reading (input data)
- File writing (compressed output)

**Requirements**:
- Temporary directories shall be created in current directory
- Temporary directory naming: `jczpkg_<random>`
- Temporary files shall be cleaned up on completion
- File permissions shall be preserved (0755 for created files)

#### 3.2.3 Data Interfaces

##### DI-001: Input File Data
**Description**: The system shall read input files as binary data streams.

**Requirements**:
- Support files of any size (limited by disk space)
- Handle binary data correctly
- Support streaming processing where possible

##### DI-002: Output File Data
**Description**: The system shall write compressed files as binary data streams.

**Requirements**:
- Use buffered I/O for efficiency
- Support streaming writes where possible
- Ensure atomic file creation (create complete file or fail)

#### 3.2.5 Encryption Requirements

##### FR-ENCRYPT-001: Password-Based Encryption
**Description**: The system shall encrypt compressed files using password-based encryption.

**Inputs**:
- Compressed file
- User password (prompted securely)

**Processing**:
1. Prompt user for password without echo
2. Generate cryptographically secure random salt (32 bytes)
3. Derive encryption key using Argon2id (64MB memory, 3 iterations)
4. Generate random nonce (12 bytes)
5. Encrypt data using AES-256-GCM
6. Create encrypted container with metadata
7. Write to file with .jcze extension

**Outputs**:
- Encrypted file (.jcze)
- Success/error status

**Error Conditions**:
- Empty password
- Encryption failure
- File write error

##### FR-ENCRYPT-002: RSA Public-Key Encryption
**Description**: The system shall encrypt compressed files using RSA public-key cryptography.

**Inputs**:
- Compressed file
- RSA public key file (PEM format, minimum 2048 bits)

**Processing**:
1. Validate and parse RSA public key
2. Generate random AES-256 symmetric key
3. Generate random nonce (12 bytes)
4. Encrypt data using AES-256-GCM with symmetric key
5. Encrypt symmetric key using RSA-OAEP-SHA256
6. Create encrypted container with encrypted key and metadata
7. Write to file with .jcze extension

**Outputs**:
- Encrypted file (.jcze)
- Success/error status

**Error Conditions**:
- Invalid or missing key file
- Key size too small (< 2048 bits)
- Invalid PEM format
- Encryption failure

##### FR-DECRYPT-001: Password-Based Decryption
**Description**: The system shall decrypt password-encrypted files.

**Inputs**:
- Encrypted file (.jcze)
- User password (prompted securely)

**Processing**:
1. Read and parse encrypted container
2. Detect password-based encryption from metadata
3. Prompt user for password
4. Derive decryption key using stored salt and Argon2 parameters
5. Decrypt data using AES-256-GCM
6. Verify authentication tag
7. Write decrypted data
8. Remove encrypted file

**Outputs**:
- Decrypted compressed file
- Success/error status

**Error Conditions**:
- Incorrect password (authentication failure)
- Corrupted encrypted file
- Invalid container format

##### FR-DECRYPT-002: RSA Private-Key Decryption
**Description**: The system shall decrypt RSA-encrypted files using private keys.

**Inputs**:
- Encrypted file (.jcze)
- RSA private key file (PEM format)

**Processing**:
1. Read and parse encrypted container
2. Validate and parse RSA private key
3. Decrypt symmetric key using RSA-OAEP-SHA256
4. Decrypt data using AES-256-GCM with recovered symmetric key
5. Verify authentication tag
6. Write decrypted data
7. Remove encrypted file

**Outputs**:
- Decrypted compressed file
- Success/error status

**Error Conditions**:
- Invalid or missing key file
- Wrong key (decryption failure)
- Corrupted encrypted file
- Authentication failure

##### FR-ENCRYPT-003: Automatic Encrypted File Detection
**Description**: The system shall automatically detect and handle encrypted files during decompression.

**Inputs**:
- File path (may be encrypted)

**Processing**:
1. Check file extension for .jcze
2. If encrypted, decrypt before decompressing
3. If not encrypted, proceed with normal decompression

**Outputs**:
- Appropriate handling based on encryption status

---

### 3.3 Non-Functional Requirements

#### 3.3.1 Performance Requirements

##### NFR-PERF-001: Concurrent Processing
**Description**: The system shall process multiple files concurrently.

**Requirements**:
- Use concurrent/parallel processing for independent file operations
- Spawn separate threads/processes per file
- Wait for all operations to complete before exit
- Report individual file errors without stopping other operations

##### NFR-PERF-002: Buffered I/O
**Description**: The system shall use buffered I/O for efficiency.

**Requirements**:
- Use buffered writers for output files
- Stream compressed data through buffers
- Minimize memory footprint for large files

##### NFR-PERF-003: Streaming Compression
**Description**: The system shall stream data during compression.

**Requirements**:
- Capture STDOUT from compression tools via pipes
- Write compressed data to output file as it becomes available
- Avoid loading entire file into memory

#### 3.3.2 Reliability Requirements

##### NFR-REL-001: Original File Preservation
**Description**: The system shall preserve original files during compression.

**Requirements**:
- Always use "keep" mode for compression tools
- Never delete original files
- Create compressed files separately from originals

##### NFR-REL-002: Temporary File Cleanup
**Description**: The system shall clean up temporary files.

**Requirements**:
- Remove intermediate files after compound compression
- Remove temporary directories after collection operations
- Use deferred cleanup to ensure execution even on errors

##### NFR-REL-003: Error Handling
**Description**: The system shall handle errors gracefully.

**Requirements**:
- Validate inputs before processing
- Capture and report tool errors
- Continue batch processing even if individual files fail
- Provide meaningful error messages
- Log errors to STDERR

#### 3.3.3 Usability Requirements

##### NFR-USE-001: Consistent Interface
**Description**: The system shall provide a consistent command-line interface.

**Requirements**:
- Single command for all compression formats
- Uniform option syntax across all operations
- Clear, descriptive option names

##### NFR-USE-002: Helpful Error Messages
**Description**: The system shall provide clear error messages.

**Requirements**:
- Include filename in error messages
- Specify nature of error (not found, invalid, etc.)
- Provide suggestions where applicable
- Display usage on invalid arguments

##### NFR-USE-003: Progress Visibility
**Description**: The system shall provide visibility into operations.

**Requirements**:
- Log operations at appropriate verbosity levels
- Show warnings for non-critical issues
- Support debug mode for troubleshooting

#### 3.3.4 Maintainability Requirements

##### NFR-MAINT-001: Modular Design
**Description**: The system shall use a modular architecture.

**Components**:
- **Core Module**: Common interfaces and utilities
- **Compression Modules**: One per algorithm (GZIP, BZIP2, XZ, TAR)
- **CLI Module**: Command-line interface and argument parsing
- **Logger Module**: Logging infrastructure

**Requirements**:
- Each compression algorithm shall have its own module
- All compression modules shall implement a common interface
- Shared functionality shall be in core utilities

##### NFR-MAINT-002: Common Configuration Interface
**Description**: Compression modules shall implement a common configuration interface.

**Interface Methods**:
- `Name()`: Return compressor name
- `Compress(infile)`: Compress a file
- `DeCompress(infile)`: Decompress a file
- `SetTimestampOption(option)`: Set timestamp option
- `SetCompLevel(level)`: Set compression level
- `SetMoveTo(to)`: Set destination directory
- `CompressMultiFiles(pkgname, infileDir)`: Compress multiple files (TAR only)

**Requirements**:
- All compressors shall implement the interface
- Methods shall have consistent signatures
- Methods shall return consistent result types

#### 3.3.5 Portability Requirements

##### NFR-PORT-001: Platform Support
**Description**: The system shall support Unix-like operating systems.

**Platforms**:
- Linux distributions
- macOS
- BSD variants
- Other POSIX-compliant systems

**Requirements**:
- Avoid platform-specific system calls
- Use standard POSIX utilities
- Handle path separators correctly

##### NFR-PORT-002: Tool Availability
**Description**: The system shall verify tool availability.

**Requirements**:
- Compression tools must be installed on system
- Tools must be in system PATH
- Provide clear error if tool not found

---

## 4. Detailed Feature Specifications

### 4.1 Configuration Data Structure

#### ConfigInfo Structure
**Purpose**: Store common configuration for all compressors.

**Fields**:
- `level`: Compression level (integer, 0-9)
- `timestampOption`: Timestamp formatting option (integer, 0-3)
- `moveto`: Destination directory path (string)
- `showOutputFileSize`: Display output file size (boolean, currently unused)

**Usage**: Each compressor configuration shall contain a ConfigInfo instance.

### 4.2 Timestamp Generation

#### Timestamp Function
**Purpose**: Generate timestamp strings for filename suffixes.

**Input**: Timestamp option (0-3)

**Output**: Formatted timestamp string

**Formats**:
- Option 0: Empty string (no timestamp)
- Option 1: `YYYYMMDD` (e.g., `20251101`)
- Option 2: `YYYYMMDD_HHMMSS` (e.g., `20251101_143522`)
- Option 3: Nanosecond component only (e.g., `123456789`)

**Requirements**:
- Use current system time
- Format integers without separators
- Pad with leading zeros where appropriate (except option 3)

### 4.3 File Path Handling

#### FileNameParse Function
**Purpose**: Split file path into parent directory and basename.

**Input**: File path (string)

**Processing**:
- Remove trailing slash if present
- Extract parent directory
- Extract basename (filename with extension)

**Output**: Tuple of (parent, basename)

**Special Cases**:
- Current directory: parent = "."
- Absolute path: parent = full directory path
- Relative path: parent = relative directory path

### 4.4 MoveTo Directory Validation

#### CheckMoveTo Function
**Purpose**: Validate destination directory for moving files.

**Input**: Directory path (string)

**Validation Steps**:
1. Check directory path is not empty
2. Check directory exists
3. Check path is a directory (not a file)

**Output**: Error or success

**Error Messages**:
- "MoveTo Directory is not specified"
- "MoveTo does not exist"
- "MoveTo X is not a directory"

### 4.5 Compression Algorithm Specifications

#### 4.5.1 GZIP Compression

**Algorithm**: DEFLATE (LZ77 + Huffman coding)

**Extension**: `.gz`

**Compression Tool**: `gzip`

**Tool Arguments**:
- Compression: `gzip -<level> --keep --stdout <infile>`
- Decompression: `gzip -d -k <infile>`

**Compression Levels**: 1-9
- 1: Fastest compression, larger files
- 9: Best compression, slower
- Default: 0 (ConfigInfo default)

**Default Behavior**:
- Keep original file (`--keep`)
- Write to STDOUT (`--stdout`) for streaming
- Capture STDOUT and write to output file

**File Naming**:
- Without timestamp: `<filename>.gz`
- With timestamp: `<filename>_<timestamp>.gz`

**Decompression**:
- Remove `.gz` extension to get output filename
- Keep original compressed file (`-k`)

**Restrictions**:
- Cannot compress directories (must error)
- Only compresses single files

#### 4.5.2 BZIP2 Compression

**Algorithm**: Burrows-Wheeler transform + Huffman coding

**Extension**: `.bz2`

**Compression Tool**: `bzip2`

**Tool Arguments**:
- Compression: `bzip2 -<level> --keep --stdout <infile>`
- Decompression: `bzip2 -d -k <infile>`

**Compression Levels**: 1-9
- 1: Fastest compression
- 9: Best compression
- Default: 0 (ConfigInfo default)

**Default Behavior**:
- Keep original file (`--keep`)
- Write to STDOUT (`--stdout`) for streaming
- Capture STDOUT and write to output file

**File Naming**:
- Without timestamp: `<filename>.bz2`
- With timestamp: `<filename>_<timestamp>.bz2`

**Decompression**:
- Remove `.bz2` extension to get output filename
- Keep original compressed file (`-k`)

**Restrictions**:
- Cannot compress directories (must error)
- Only compresses single files

#### 4.5.3 XZ Compression

**Algorithm**: LZMA/LZMA2

**Extension**: `.xz`

**Compression Tool**: `xz`

**Tool Arguments**:
- Compression: `xz -<level> --keep --stdout <infile>`
- Decompression: `xz -d -k <infile>`

**Compression Levels**: 0-9
- 0: Fastest compression
- 9: Best compression
- Default: 6 (ConfigInfo default)

**Default Behavior**:
- Keep original file (`--keep`)
- Write to STDOUT (`--stdout`) for streaming
- Capture STDOUT and write to output file

**File Naming**:
- Without timestamp: `<filename>.xz`
- With timestamp: `<filename>_<timestamp>.xz`

**Decompression**:
- Remove `.xz` extension to get output filename
- Keep original compressed file (`-k`)

**Restrictions**:
- Cannot compress directories (must error)
- Only compresses single files

#### 4.5.4 ZIP Compression

**Algorithm**: DEFLATE (similar to GZIP) with archive support

**Extension**: `.zip`

**Compression Tool**: `zip` (compression), `unzip` (decompression)

**Tool Arguments**:
- Compression: `zip -<level> -q [-r] <outfile> <infile>`
- Decompression: `unzip -o <infile> -d <outdir>`

**Compression Levels**: 0-9
- 0: Store only (no compression)
- 1: Fastest compression
- 9: Best compression
- Default: 6

**Default Behavior**:
- Keep original file (zip doesn't modify source)
- Recursive flag (`-r`) for directories
- Quiet mode (`-q`) to suppress output
- Overwrite without prompting (`-o`) for decompression

**File Naming**:
- Without timestamp: `<filename>.zip`
- With timestamp: `<filename>_<timestamp>.zip`

**Decompression**:
- Extract to parent directory or specified destination
- Remove `.zip` extension to determine output name
- Supports single files, directories, and multiple files

**Special Features**:
- Can compress both files and directories
- Archive format (can contain multiple files)
- Cross-platform compatibility

**Restrictions**:
- None (supports both files and directories)

#### 4.5.5 TAR Archiving

**Format**: POSIX ustar format

**Extension**: `.tar`

**Archiving Tool**: `tar`

**Tool Arguments**:
- Single file/directory: `tar -C <parent> -cf <outfile> <basename>`
- Multiple files (collection): `tar -C <infileDir> -cf <outfile> <file1> <file2> ...`
- Extraction: `tar -x -C <parent> -f <infile>`

**Compression Level**:
- Not applicable (archiving only, not compression)
- SetCompLevel returns true (no-op)

**Default Behavior**:
- Create archive from files/directories
- Preserve directory structure
- Support both single and multiple file archiving

**File Naming**:
- Without timestamp: `<filename>.tar`
- With timestamp: `<filename>_<timestamp>.tar`
- Strip trailing slash from input path before adding extension

**Decompression**:
- Extract to parent directory of archive file
- Remove `.tar` extension to get extracted directory name

**Special Capabilities**:
- **CompressMultiFiles**: Archive multiple files without parent directory wrapper
  - Create TAR from contents of directory
  - Use `tar -C <dir> -cf <output> <file1> <file2> ...`
  - List directory contents and add each to archive

### 4.6 Compound Format Handling

#### 4.6.1 TGZ (TAR + GZIP)

**Extension**: `.tar.gz`

**Process**:
1. Create TAR archive from input file(s)
2. Compress TAR file with GZIP
3. Remove intermediate TAR file
4. Move final .tar.gz to destination if specified

**Compression Level**: Applies to GZIP step only

**Timestamp**: Applies to TAR step only (TAR filename gets timestamp)

**Example Flow**:
- Input: `mydir/`
- TAR output: `mydir.tar` or `mydir_20251101.tar`
- GZIP output: `mydir.tar.gz` or `mydir_20251101.tar.gz`

#### 4.6.2 TBZ2 (TAR + BZIP2)

**Extension**: `.tar.bz2`

**Process**:
1. Create TAR archive from input file(s)
2. Compress TAR file with BZIP2
3. Remove intermediate TAR file
4. Move final .tar.bz2 to destination if specified

**Compression Level**: Applies to BZIP2 step only

**Timestamp**: Applies to TAR step only

#### 4.6.3 TXZ (TAR + XZ)

**Extension**: `.tar.xz`

**Process**:
1. Create TAR archive from input file(s)
2. Compress TAR file with XZ
3. Remove intermediate TAR file
4. Move final .tar.xz to destination if specified

**Compression Level**: Applies to XZ step only

**Timestamp**: Applies to TAR step only

### 4.7 Collection Compression Algorithm

#### Purpose
Combine multiple files/directories into a single compressed archive.

#### Input
- Multiple file/directory paths
- Package name (output archive base name)
- Compression format (tar, tgz, tbz2, txz)
- Optional: timestamp, compression level, destination

#### Process
1. **Validation**:
   - Verify package name is not empty
   - Verify package name doesn't exist in current directory
   - Verify at least one input file
   - Verify no duplicate basenames

2. **Staging**:
   - Create temporary directory: `jczpkg_<random>` in current directory
   - Create subdirectory: `<tmpdir>/<pkgname>`
   - Copy all input files to staging directory: `cp -r <file> <staging>/`

3. **Archiving**:
   - **Standard mode (-a)**: Compress staging directory with parent:
     - TAR the `<pkgname>` directory
     - Result includes parent directory in archive
   - **Flat mode (-A)**: Compress staging contents without parent:
     - TAR the contents of `<pkgname>` directory
     - Result has files at archive root

4. **Optional Secondary Compression**:
   - If format is tgz/tbz2/txz:
     - Compress the TAR file with appropriate algorithm
     - Remove intermediate TAR file

5. **Finalization**:
   - Move final archive to destination or current directory
   - Clean up temporary directory

6. **Cleanup**:
   - Remove temporary directory and all contents
   - Use deferred cleanup to ensure execution even on errors

#### Error Handling
- Package name exists: Return error
- Duplicate basenames: Return error (cannot collect files with same name)
- Copy failure: Return error
- TAR failure: Return error, cleanup temp directory
- Compression failure: Return error, cleanup temp directory

### 4.8 Batch Processing Algorithm

#### Purpose
Process multiple files concurrently with parallelism.

#### Compression Flow
1. Validate all input files
2. Deduplicate input files (by resolved path)
3. Detect duplicate basenames
4. Create compressor configuration
5. For each input file:
   - Spawn concurrent operation
   - Compress file using configured compressor
   - Log errors individually
   - Continue processing other files even if one fails
6. Wait for all operations to complete
7. Return aggregate status

#### Decompression Flow
1. Validate all input files have valid extensions
2. Deduplicate input files
3. For each input file:
   - Spawn concurrent operation
   - Detect compression format(s)
   - Decompress sequentially through all layers
   - Log errors individually
   - Continue processing other files even if one fails
4. Wait for all operations to complete
5. Return aggregate status

#### Concurrency Requirements
- Use goroutines/async tasks/threads for parallelism
- One concurrent task per input file
- Synchronization: Wait for all tasks before exit
- Error isolation: Individual file errors don't stop other files
- Error reporting: Log each file error separately

---

## 5. Use Cases

### 5.1 Compress Single File with GZIP
**Actor**: User
**Goal**: Compress a file using GZIP

**Preconditions**:
- Input file exists
- gzip tool is installed

**Main Flow**:
1. User executes: `jcz -c gzip myfile.txt`
2. System validates input file
3. System creates compressor configuration
4. System compresses file to `myfile.txt.gz`
5. System logs success
6. System exits with code 0

**Postconditions**:
- Original file preserved
- Compressed file created

### 5.2 Create Compressed Archive of Multiple Files
**Actor**: User
**Goal**: Combine multiple files into a compressed archive

**Preconditions**:
- Input files exist
- No duplicate basenames among inputs
- tar and gzip tools installed

**Main Flow**:
1. User executes: `jcz -c tgz -a myarchive file1.txt file2.txt dir1/`
2. System validates input files
3. System checks for duplicate basenames
4. System creates temporary directory
5. System copies files to staging area
6. System creates TAR archive
7. System compresses TAR with GZIP
8. System moves final archive to current directory
9. System cleans up temporary files
10. System exits with code 0

**Postconditions**:
- Archive file `myarchive.tar.gz` created
- Original files preserved
- Temporary files removed

### 5.3 Decompress Compound Archive
**Actor**: User
**Goal**: Extract files from a .tar.gz archive

**Preconditions**:
- Compressed archive exists
- gzip and tar tools installed

**Main Flow**:
1. User executes: `jcz -d archive.tar.gz`
2. System detects .gz extension
3. System decompresses GZIP layer to `archive.tar`
4. System detects .tar extension
5. System extracts TAR archive
6. System removes intermediate `archive.tar` file
7. System exits with code 0

**Postconditions**:
- Files extracted to current directory
- Original archive preserved
- Intermediate files removed

### 5.4 Compress with Timestamp and Move
**Actor**: User
**Goal**: Compress file with timestamp and move to backup directory

**Preconditions**:
- Input file exists
- Destination directory exists and is writable
- bzip2 tool installed

**Main Flow**:
1. User executes: `jcz -c bzip2 -t 2 -C /backups/ important.doc`
2. System validates input file
3. System validates destination directory
4. System generates timestamp (e.g., 20251101_143522)
5. System compresses to `important.doc_20251101_143522.bz2`
6. System moves compressed file to `/backups/`
7. System exits with code 0

**Postconditions**:
- Compressed file in `/backups/` with timestamp
- Original file preserved in original location

### 5.5 Batch Compress Multiple Files
**Actor**: User
**Goal**: Compress multiple files concurrently

**Preconditions**:
- Input files exist
- xz tool installed

**Main Flow**:
1. User executes: `jcz -c xz -l 9 file1.txt file2.txt file3.txt`
2. System validates input files
3. System creates XZ compressor with level 9
4. System spawns 3 concurrent compression operations
5. Each operation compresses its file independently
6. System waits for all operations to complete
7. System exits with code 0

**Postconditions**:
- Three .xz files created
- Original files preserved

---

## 6. Data Requirements

### 6.1 Configuration Data

#### Compressor Configuration
- Compression level (integer)
- Timestamp option (integer)
- Destination directory (string)
- Output file size display flag (boolean)

#### CLI Arguments
- Decompress flag (boolean)
- Compression command (string)
- Compression level (integer)
- Destination directory (string)
- Collection package name (string)
- Collection mode (standard/flat)
- Timestamp option (integer)
- Input file list (array of strings)

### 6.2 File Metadata

#### Input File Information
- Absolute file path
- File type (regular file, directory, symbolic link)
- File size
- File permissions
- File existence status

#### Output File Information
- Generated filename
- File path
- File size
- Destination path

### 6.3 Logging Data

#### Log Entries
- Log level (ERROR, WARN, INFO, DEBUG)
- Timestamp (automatic by logger)
- Source file location (for ERROR and DEBUG)
- Message text

---

## 7. Quality Attributes

### 7.1 Correctness
- Compressed files shall be valid and decompressible
- Decompressed files shall match original content exactly
- File metadata shall be preserved where applicable

### 7.2 Robustness
- Tool shall handle errors gracefully without crashes
- Invalid inputs shall be rejected with clear error messages
- Resource leaks shall be prevented (file handles, memory, temp files)

### 7.3 Efficiency
- Parallel processing shall improve throughput for multiple files
- Streaming I/O shall minimize memory usage for large files
- Buffered I/O shall reduce system call overhead

### 7.4 Usability
- Command-line interface shall be intuitive
- Error messages shall be helpful and actionable
- Logging shall provide visibility into operations

### 7.5 Maintainability
- Modular architecture shall support adding new compression formats
- Common interfaces shall reduce code duplication
- Clear separation between CLI and core logic

---

## 8. Constraints and Assumptions

### 8.1 Constraints
- Must use external system compression tools (cannot implement compression algorithms)
- Must operate in command-line environment
- Must run on Unix-like operating systems
- Must preserve original files during compression
- Must support POSIX filesystem semantics

### 8.2 Assumptions
- System compression tools (gzip, bzip2, xz, tar) are installed
- System utilities (mv, cp, readlink) are available
- Users have appropriate file system permissions
- File paths are valid and within system limits
- Sufficient disk space for compression operations
- Input files are not corrupted
- Temporary directory creation is allowed in current directory

---

## 9. Appendices

### Appendix A: Compression Format Comparison

| Format | Algorithm | Ratio | Speed | Level Range | Use Case |
|--------|-----------|-------|-------|-------------|----------|
| GZIP | DEFLATE | Good | Fast | 1-9 | General purpose, widely compatible |
| BZIP2 | BWT+Huffman | Better | Slower | 1-9 | Better compression, slower |
| XZ | LZMA2 | Best | Slowest | 0-9 | Best compression, resource intensive |
| TAR | None | N/A | Fastest | N/A | Archiving only, no compression |

### Appendix B: File Extension Mapping

| Extension | Format | Compression | Archive |
|-----------|--------|-------------|---------|
| .gz | GZIP | Yes | No |
| .bz2 | BZIP2 | Yes | No |
| .xz | XZ | Yes | No |
| .tar | TAR | No | Yes |
| .tar.gz / .tgz | TAR+GZIP | Yes | Yes |
| .tar.bz2 / .tbz2 | TAR+BZIP2 | Yes | Yes |
| .tar.xz / .txz | TAR+XZ | Yes | Yes |

### Appendix C: Error Code Reference

| Exit Code | Meaning |
|-----------|---------|
| 0 | Success |
| 1 | Error (general) |

**Note**: The current implementation uses only 0 and 1. Future versions could implement more granular error codes.

### Appendix D: Environment Variables

| Variable | Values | Default | Purpose |
|----------|--------|---------|---------|
| JCZDBG | error, warn, info, debug | info | Control logging verbosity |

### Appendix E: Temporary File Naming

| Type | Pattern | Example |
|------|---------|---------|
| Staging directory | `jczpkg_<random>` | `jczpkg_a8f3d2b1` |
| Intermediate TAR | `<original>.tar` | `myfile.txt.tar` |

### Appendix F: Command Examples

```bash
# Basic compression
jcz -c gzip myfile.txt

# Compression with level
jcz -c bzip2 -l 9 large.iso

# Create compressed archive
jcz -c tgz -a backup file1 file2 dir/

# Create flat archive (no parent directory)
jcz -c txz -A backup file1 file2 dir/

# Compress with timestamp
jcz -c xz -t 2 document.pdf

# Compress and move to directory
jcz -c gzip -C /backups/ important.doc

# Compress with all options
jcz -c bzip2 -l 9 -t 2 -C /archives/ file.bin

# Decompress single file
jcz -d archive.tar.gz

# Decompress and move
jcz -d -C /extracted/ archive.tar.xz

# Batch compress
jcz -c gzip *.txt

# Batch decompress
jcz -d *.gz

# Enable debug logging
JCZDBG=debug jcz -c gzip myfile.txt
```

---

**End of SRS Document**
