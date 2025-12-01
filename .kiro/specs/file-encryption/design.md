# Design Document: File Encryption Feature

## Overview

This design document describes the implementation of encryption capabilities for the JCZ compression utility. The feature adds support for encrypting compressed files using either password-based encryption (AES-256-GCM with Argon2id key derivation) or RSA public-key cryptography. The design integrates seamlessly with the existing compression/decompression workflow while maintaining the modular architecture of the JCZ system.

The encryption layer operates on already-compressed data, wrapping it in an encrypted container with metadata. This approach allows encryption to work uniformly across all compression formats without modifying individual compressor implementations.

## Architecture

### High-Level Design

The encryption feature follows a layered architecture:

1. **CLI Layer**: Extended command-line arguments for encryption options
2. **Encryption Module**: Core encryption/decryption logic with support for multiple methods
3. **Integration Layer**: Hooks into existing compress/decompress operations
4. **Cryptography Layer**: Low-level cryptographic primitives using the `ring` and `rsa` crates

### Data Flow

**Compression with Encryption:**
```
Input File → Compress → Encrypted Container → Output File
                ↓
         [Encryption Metadata]
```

**Decompression with Decryption:**
```
Encrypted File → Read Metadata → Decrypt → Decompress → Output File
```

### Module Structure

```
src/
├── crypto/
│   ├── mod.rs              # Public API and types
│   ├── password.rs         # Password-based encryption
│   ├── rsa.rs              # RSA encryption
│   ├── container.rs        # Encrypted container format
│   └── keys.rs             # Key management utilities
├── core/
│   ├── config.rs           # Extended with encryption config
│   ├── error.rs            # Extended with crypto errors
│   └── types.rs            # Extended with encryption types
└── operations/
    ├── encrypt.rs          # Encryption operations
    └── decrypt.rs          # Decryption operations
```

## Components and Interfaces

### 1. Encryption Configuration

```rust
/// Encryption method selection
pub enum EncryptionMethod {
    Password,
    Rsa { private_key_path: PathBuf },
}

/// Decryption method selection
pub enum DecryptionMethod {
    Password,
    Rsa { public_key_path: PathBuf },
}

/// Extended compression configuration with encryption
pub struct CompressionConfig {
    // ... existing fields ...
    pub encryption: Option<EncryptionMethod>,
}

/// Extended decompression configuration
pub struct DecompressionConfig {
    // ... existing fields ...
    pub decryption: Option<DecryptionMethod>,
}
```

### 2. Encrypted Container Format

The encrypted file format consists of:

```
[Magic Bytes: 4 bytes] "JCZE"
[Version: 1 byte]
[Encryption Type: 1 byte] (0x01 = Password, 0x02 = RSA)
[Metadata Length: 4 bytes]
[Metadata: variable]
[Encrypted Data: variable]
```

**Password Encryption Metadata:**
```
[Salt: 32 bytes]
[Nonce: 12 bytes]
[Argon2 Parameters: variable]
```

**RSA Encryption Metadata:**
```
[Encrypted Symmetric Key Length: 4 bytes]
[Encrypted Symmetric Key: variable]
[Nonce: 12 bytes]
```

### 3. Cryptography Interfaces

```rust
/// Password-based encryption
pub trait PasswordEncryption {
    fn encrypt(&self, data: &[u8], password: &str) -> Result<Vec<u8>, CryptoError>;
    fn decrypt(&self, data: &[u8], password: &str) -> Result<Vec<u8>, CryptoError>;
}

/// RSA encryption
pub trait RsaEncryption {
    fn encrypt_with_private_key(
        &self,
        data: &[u8],
        private_key: &RsaPrivateKey,
    ) -> Result<Vec<u8>, CryptoError>;
    
    fn decrypt_with_public_key(
        &self,
        data: &[u8],
        public_key: &RsaPublicKey,
    ) -> Result<Vec<u8>, CryptoError>;
}

/// Container operations
pub trait EncryptedContainer {
    fn write(&self, output: &Path) -> Result<(), CryptoError>;
    fn read(input: &Path) -> Result<Self, CryptoError>;
    fn get_encryption_type(&self) -> EncryptionType;
}
```

### 4. CLI Extensions

```rust
#[derive(Parser, Debug)]
pub struct CliArgs {
    // ... existing fields ...
    
    /// Enable password-based encryption
    #[arg(long = "encrypt-password", short = 'e')]
    pub encrypt_password: bool,
    
    /// RSA private key file for encryption
    #[arg(long = "encrypt-rsa")]
    pub encrypt_rsa: Option<PathBuf>,
    
    /// RSA public key file for decryption
    #[arg(long = "decrypt-rsa")]
    pub decrypt_rsa: Option<PathBuf>,
}
```

## Data Models

### EncryptedContainer

```rust
pub struct EncryptedContainer {
    pub version: u8,
    pub encryption_type: EncryptionType,
    pub metadata: EncryptionMetadata,
    pub encrypted_data: Vec<u8>,
}

pub enum EncryptionType {
    Password,
    Rsa,
}

pub enum EncryptionMetadata {
    Password {
        salt: [u8; 32],
        nonce: [u8; 12],
        argon2_params: Argon2Params,
    },
    Rsa {
        encrypted_key: Vec<u8>,
        nonce: [u8; 12],
    },
}

pub struct Argon2Params {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}
```

### Error Types

```rust
pub enum CryptoError {
    InvalidPassword,
    InvalidKey,
    KeyDerivationFailed,
    EncryptionFailed,
    DecryptionFailed,
    AuthenticationFailed,
    InvalidContainer,
    UnsupportedVersion(u8),
    IoError(io::Error),
    RsaError(String),
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Property Reflection

After analyzing all acceptance criteria, several properties can be consolidated:

**Consolidations:**
- Properties 1.3, 3.4, and 7.3 all relate to AES-256-GCM usage - can be verified through round-trip testing
- Properties 2.4, 4.4, and 4.5 all relate to RSA round-trip - can be combined into one comprehensive property
- Properties 1.4 and 2.5 both test metadata storage - can be unified
- Properties 3.3 and 1.2 both test key derivation - can be combined
- Properties 4.1 and 2.1 both test file validation - can be unified
- Properties 4.2 and 2.2 both test PEM parsing - can be unified
- Properties 5.1 and 5.2 describe the integration workflow - can be tested as one property

**Core Properties to Implement:**
1. Password encryption round-trip (covers 1.2, 1.3, 1.4, 3.3, 3.4)
2. RSA encryption round-trip (covers 2.3, 2.4, 2.5, 4.3, 4.4, 4.5)
3. Encryption metadata completeness (covers 1.4, 2.5, 7.4)
4. Key file validation (covers 2.1, 4.1)
5. PEM key parsing (covers 2.2, 4.2)
6. Wrong password authentication failure (3.5)
7. Compression options preservation (5.3)
8. Independent file encryption (5.5)
9. Random value uniqueness (7.2)
10. RSA key size and padding validation (7.5)
11. Non-encrypted file handling (8.5)

### Correctness Properties

**Property 1: Password encryption round-trip**

*For any* compressed file and any non-empty password, encrypting the file with that password and then decrypting with the same password should produce data that decompresses to the original content.

**Validates: Requirements 1.2, 1.3, 1.4, 3.3, 3.4**

---

**Property 2: RSA encryption round-trip**

*For any* compressed file and any valid RSA key pair (private key for encryption, public key for decryption), encrypting the file with the private key and then decrypting with the corresponding public key should produce data that decompresses to the original content.

**Validates: Requirements 2.3, 2.4, 2.5, 4.3, 4.4, 4.5**

---

**Property 3: Encryption metadata completeness**

*For any* encrypted file (password or RSA), the stored metadata should contain all parameters necessary for decryption (salt, nonce, Argon2 parameters for password; encrypted key and nonce for RSA) and should not contain the actual encryption key or password.

**Validates: Requirements 1.4, 2.5, 7.4**

---

**Property 4: Key file validation**

*For any* file path provided as a key file, if the file does not exist or is not readable, the system should reject the operation with an appropriate error before attempting any cryptographic operations.

**Validates: Requirements 2.1, 4.1**

---

**Property 5: PEM key parsing**

*For any* valid PEM-formatted RSA key file (private or public), the system should successfully parse the key and be able to use it for cryptographic operations.

**Validates: Requirements 2.2, 4.2**

---

**Property 6: Wrong password authentication failure**

*For any* password-encrypted file, attempting to decrypt with an incorrect password should fail authentication during the AES-GCM decryption process and return a clear error message.

**Validates: Requirements 3.5**

---

**Property 7: Compression options preservation**

*For any* compression operation with encryption enabled and any combination of compression options (level, timestamp, move-to directory), the resulting file should reflect all specified compression options as if encryption were not enabled.

**Validates: Requirements 5.3**

---

**Property 8: Independent file encryption**

*For any* set of multiple files being compressed and encrypted, each file should be encrypted with an independent symmetric key and nonce, such that compromising one file's encryption does not affect others.

**Validates: Requirements 5.5**

---

**Property 9: Random value uniqueness**

*For any* sequence of encryption operations, generated random values (nonces, symmetric keys, salts) should be unique across operations with overwhelming probability.

**Validates: Requirements 7.2**

---

**Property 10: RSA key size and padding validation**

*For any* RSA key provided for encryption or decryption, the system should verify the key size is at least 2048 bits and should use OAEP padding for all RSA operations.

**Validates: Requirements 7.5**

---

**Property 11: Non-encrypted file handling**

*For any* non-encrypted compressed file, providing encryption/decryption options during decompression should not cause errors and should proceed with normal decompression.

**Validates: Requirements 8.5**

---

**Property 12: Encrypted file detection**

*For any* file processed by the system, the system should correctly identify whether it is encrypted by reading the magic bytes and metadata header.

**Validates: Requirements 3.1**

---

**Property 13: File extension indication**

*For any* encrypted compressed file, the output filename should include an extension that indicates encryption (e.g., .jcze or .enc) in addition to the compression format extension.

**Validates: Requirements 5.4**

---

**Property 14: Error message specificity**

*For any* encryption or decryption error condition (wrong password, invalid key, corrupted data, etc.), the system should provide an error message that specifically identifies the type of failure.

**Validates: Requirements 6.5**

## Error Handling

### Error Categories

1. **Input Validation Errors**
   - Empty password
   - Missing key files
   - Invalid key file format
   - Conflicting encryption options

2. **Cryptographic Errors**
   - Key derivation failure
   - Encryption/decryption failure
   - Authentication failure (wrong password/key)
   - Invalid key size

3. **Container Format Errors**
   - Invalid magic bytes
   - Unsupported version
   - Corrupted metadata
   - Truncated encrypted data

4. **I/O Errors**
   - File read/write failures
   - Permission errors
   - Disk space issues

### Error Handling Strategy

- All cryptographic operations return `Result<T, CryptoError>`
- Errors are propagated up to the CLI layer for user-friendly display
- Sensitive information (passwords, keys) is never included in error messages
- Failed operations clean up temporary files automatically
- Authentication failures are distinguished from other decryption errors

## Testing Strategy

### Unit Testing

Unit tests will cover:

- Container format serialization/deserialization
- Key derivation with known test vectors
- Metadata parsing and validation
- Error condition handling (empty passwords, invalid keys)
- CLI argument validation
- File extension generation

### Property-Based Testing

Property-based testing will be implemented using the `proptest` crate for Rust. Each correctness property will be tested with a minimum of 100 randomly generated test cases.

**Property Test Configuration:**
```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    // ... test cases
}
```

**Test Generators:**

1. **Random Data Generator**: Generates arbitrary byte sequences for file content
2. **Password Generator**: Generates valid passwords of varying lengths and character sets
3. **RSA Key Pair Generator**: Generates valid RSA key pairs with different key sizes
4. **Compression Config Generator**: Generates random compression configurations
5. **File Path Generator**: Generates valid and invalid file paths

**Property Test Tags:**

Each property-based test will be tagged with a comment referencing the design document:

```rust
// Feature: file-encryption, Property 1: Password encryption round-trip
#[test]
fn prop_password_round_trip() { ... }
```

### Integration Testing

Integration tests will verify:

- End-to-end encryption/decryption workflows
- Integration with all compression formats (gzip, bzip2, xz, zip, tar, compound)
- CLI argument parsing and execution
- File system operations (move-to directory, timestamps)
- Parallel processing of multiple encrypted files

### Security Testing

Security-focused tests will verify:

- Argon2id parameters meet security recommendations
- Random number generation uses cryptographically secure sources
- Sensitive data is properly zeroized after use
- Timing attacks are mitigated where possible
- Key sizes meet minimum security requirements

## Implementation Notes

### Dependencies

New dependencies to add to `Cargo.toml`:

```toml
# Cryptography
ring = "0.17"           # AES-GCM, CSPRNG
rsa = "0.9"             # RSA operations
argon2 = "0.5"          # Password hashing
pem = "3.0"             # PEM file parsing
zeroize = "1.7"         # Secure memory clearing

# Property testing
[dev-dependencies]
proptest = "1.4"        # Property-based testing
```

### Security Considerations

1. **Password Handling**
   - Passwords are read from stdin without echo
   - Passwords are zeroized immediately after key derivation
   - No password caching or storage

2. **Key Material**
   - Symmetric keys are generated using `ring::rand::SystemRandom`
   - Keys are zeroized after use using the `zeroize` crate
   - Private keys are never written to disk by the application

3. **Argon2id Parameters**
   - Memory cost: 64 MB (65536 KB)
   - Time cost: 3 iterations
   - Parallelism: 4 threads
   - These parameters provide strong resistance to brute-force attacks

4. **RSA Configuration**
   - Minimum key size: 2048 bits (recommended: 4096 bits)
   - Padding: OAEP with SHA-256
   - Only the symmetric key is encrypted with RSA, not the entire file

### Performance Considerations

1. **Parallel Processing**
   - Encryption/decryption operations can be parallelized across multiple files
   - Individual file encryption is sequential (AES-GCM is not easily parallelizable)

2. **Memory Usage**
   - Files are processed in streaming mode where possible
   - Argon2id memory cost is configurable but defaults to 64 MB
   - RSA operations are performed only on small symmetric keys (32 bytes)

3. **Overhead**
   - Encryption adds minimal overhead to file size (< 1 KB metadata)
   - Performance impact is primarily from AES-GCM encryption (typically > 1 GB/s on modern CPUs)

### File Extension Convention

Encrypted files will use the following naming convention:

- Password-encrypted: `filename.ext.jcze` (e.g., `data.tar.gz.jcze`)
- RSA-encrypted: `filename.ext.jcze` (same extension, type stored in metadata)

The `.jcze` extension stands for "JCZ Encrypted" and clearly indicates the file requires decryption.

### Backward Compatibility

- Existing compressed files without encryption remain fully compatible
- The system automatically detects encrypted vs. non-encrypted files
- No changes to existing compression/decompression workflows
- Encryption is opt-in via CLI flags

## Future Enhancements

Potential future improvements:

1. **Additional Encryption Methods**
   - ChaCha20-Poly1305 as an alternative to AES-GCM
   - Support for hardware security modules (HSM)

2. **Key Management**
   - SSH key integration
   - Key agent support
   - Multiple recipient support (encrypt for multiple public keys)

3. **Compression Integration**
   - Encrypt-then-compress vs. compress-then-encrypt options
   - Streaming encryption for very large files

4. **Advanced Features**
   - Digital signatures for authenticity verification
   - Key rotation capabilities
   - Encrypted archive inspection without decryption
