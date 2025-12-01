//! Cryptography module for file encryption and decryption
//!
//! This module provides encryption capabilities for compressed files using:
//! - Password-based encryption with AES-256-GCM and Argon2id key derivation
//! - RSA public-key encryption with OAEP padding

pub mod container;
pub mod keys;
pub mod password;
pub mod rsa;

use std::path::PathBuf;

// Re-export commonly used types
pub use container::EncryptedContainer;
pub use password::PasswordEncryption;
pub use rsa::RsaEncryption;

/// Encryption type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionType {
    /// Password-based encryption (0x01)
    Password = 0x01,
    /// RSA encryption (0x02)
    Rsa = 0x02,
}

impl EncryptionType {
    /// Convert from byte value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(EncryptionType::Password),
            0x02 => Some(EncryptionType::Rsa),
            _ => None,
        }
    }

    /// Convert to byte value
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Argon2 parameters for key derivation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Argon2Params {
    /// Memory cost in KB (default: 65536 = 64 MB)
    pub memory_cost: u32,
    /// Time cost (iterations, default: 3)
    pub time_cost: u32,
    /// Parallelism (threads, default: 4)
    pub parallelism: u32,
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            memory_cost: 65536, // 64 MB
            time_cost: 3,
            parallelism: 4,
        }
    }
}

/// Encryption metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionMetadata {
    /// Password-based encryption metadata
    Password {
        salt: [u8; 32],
        nonce: [u8; 12],
        argon2_params: Argon2Params,
    },
    /// RSA encryption metadata
    Rsa {
        encrypted_key: Vec<u8>,
        nonce: [u8; 12],
    },
}

/// Cryptography error types
#[derive(Debug)]
#[allow(dead_code)]
pub enum CryptoError {
    /// Invalid or empty password
    InvalidPassword,
    /// Invalid key
    InvalidKey,
    /// Key derivation failed
    KeyDerivationFailed(String),
    /// Encryption operation failed
    EncryptionFailed(String),
    /// Decryption operation failed
    DecryptionFailed(String),
    /// Authentication failed (wrong password/key)
    AuthenticationFailed,
    /// Invalid container format
    InvalidContainer(String),
    /// Unsupported container version
    UnsupportedVersion(u8),
    /// I/O error
    IoError(std::io::Error),
    /// RSA error
    RsaError(String),
    /// Key file not found
    KeyFileNotFound(PathBuf),
    /// Key file not readable
    KeyFileNotReadable(PathBuf),
    /// Invalid PEM format
    InvalidPemFormat(String),
    /// Key size too small
    KeySizeTooSmall { actual: usize, minimum: usize },
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::InvalidPassword => write!(f, "Invalid or empty password"),
            CryptoError::InvalidKey => write!(f, "Invalid encryption key"),
            CryptoError::KeyDerivationFailed(msg) => write!(f, "Key derivation failed: {}", msg),
            CryptoError::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
            CryptoError::DecryptionFailed(msg) => write!(f, "Decryption failed: {}", msg),
            CryptoError::AuthenticationFailed => {
                write!(
                    f,
                    "Authentication failed: incorrect password or corrupted data"
                )
            }
            CryptoError::InvalidContainer(msg) => write!(f, "Invalid container format: {}", msg),
            CryptoError::UnsupportedVersion(v) => {
                write!(f, "Unsupported container version: {}", v)
            }
            CryptoError::IoError(err) => write!(f, "I/O error: {}", err),
            CryptoError::RsaError(msg) => write!(f, "RSA error: {}", msg),
            CryptoError::KeyFileNotFound(path) => {
                write!(f, "Key file not found: {}", path.display())
            }
            CryptoError::KeyFileNotReadable(path) => {
                write!(f, "Key file not readable: {}", path.display())
            }
            CryptoError::InvalidPemFormat(msg) => write!(f, "Invalid PEM format: {}", msg),
            CryptoError::KeySizeTooSmall { actual, minimum } => write!(
                f,
                "Key size too small: {} bits (minimum: {} bits)",
                actual, minimum
            ),
        }
    }
}

impl std::error::Error for CryptoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CryptoError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        CryptoError::IoError(err)
    }
}

/// Result type for crypto operations
pub type CryptoResult<T> = Result<T, CryptoError>;
