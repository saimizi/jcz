//! Encryption operations for compressed files

use crate::core::config::EncryptionMethod;
use crate::core::error::{JcError, JcResult};
use crate::crypto::{
    Argon2Params, EncryptedContainer, EncryptionMetadata, EncryptionType, PasswordEncryption,
    RsaEncryption,
};
use crate::utils::{error, info};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Prompt user for password securely (without echo)
fn prompt_password() -> JcResult<String> {
    use std::io::{self, Write};

    print!("Enter encryption password: ");
    io::stdout().flush()?;

    // Read password without echo
    let password = rpassword::read_password()
        .map_err(|e| JcError::Other(format!("Failed to read password: {}", e)))?;

    if password.is_empty() {
        return Err(JcError::Other("Password cannot be empty".to_string()));
    }

    Ok(password)
}

/// Encrypt a single compressed file
pub fn encrypt_file(
    compressed_file: &Path,
    encryption_method: &EncryptionMethod,
) -> JcResult<PathBuf> {
    info!("Encrypting file: {}", compressed_file.display());

    // Read the compressed data
    let compressed_data = fs::read(compressed_file)?;

    // Encrypt based on method
    let (encryption_type, metadata, encrypted_data) = match encryption_method {
        EncryptionMethod::Password => {
            // Prompt for password
            let password = prompt_password()?;

            // Generate salt and nonce
            let salt = PasswordEncryption::generate_salt()
                .map_err(|e| JcError::Other(format!("Failed to generate salt: {}", e)))?;
            let nonce = PasswordEncryption::generate_nonce()
                .map_err(|e| JcError::Other(format!("Failed to generate nonce: {}", e)))?;

            // Derive key from password
            let params = Argon2Params::default();
            let key = PasswordEncryption::derive_key(&password, &salt, &params)
                .map_err(|e| JcError::Other(format!("Key derivation failed: {}", e)))?;

            // Encrypt data
            let encrypted = PasswordEncryption::encrypt(&compressed_data, &key, &nonce)
                .map_err(|e| JcError::Other(format!("Encryption failed: {}", e)))?;

            let metadata = EncryptionMetadata::Password {
                salt,
                nonce,
                argon2_params: params,
            };

            (EncryptionType::Password, metadata, encrypted)
        }
        EncryptionMethod::Rsa { public_key_path } => {
            // Generate symmetric key and nonce
            let symmetric_key = RsaEncryption::generate_symmetric_key()
                .map_err(|e| JcError::Other(format!("Failed to generate symmetric key: {}", e)))?;
            let nonce = RsaEncryption::generate_nonce()
                .map_err(|e| JcError::Other(format!("Failed to generate nonce: {}", e)))?;

            // Encrypt data with symmetric key
            let encrypted_data =
                RsaEncryption::encrypt_data(&compressed_data, &symmetric_key, &nonce)
                    .map_err(|e| JcError::Other(format!("Data encryption failed: {}", e)))?;

            // Encrypt symmetric key with RSA public key
            let encrypted_key =
                RsaEncryption::encrypt_symmetric_key(&symmetric_key, public_key_path)
                    .map_err(|e| JcError::Other(format!("RSA encryption failed: {}", e)))?;

            let metadata = EncryptionMetadata::Rsa {
                encrypted_key,
                nonce,
            };

            (EncryptionType::Rsa, metadata, encrypted_data)
        }
    };

    // Create encrypted container
    let container = EncryptedContainer::new(encryption_type, metadata, encrypted_data);

    // Generate output filename with .jcze extension
    let output_path = compressed_file.with_extension(format!(
        "{}.jcze",
        compressed_file
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    ));

    // Write encrypted container
    container
        .write_to_file(&output_path)
        .map_err(|e| JcError::Other(format!("Failed to write encrypted file: {}", e)))?;

    info!("Encrypted file created: {}", output_path.display());

    // Remove original compressed file
    fs::remove_file(compressed_file)?;

    Ok(output_path)
}

/// Encrypt multiple compressed files in parallel
pub fn encrypt_files(
    compressed_files: Vec<PathBuf>,
    encryption_method: &EncryptionMethod,
) -> Vec<JcResult<PathBuf>> {
    info!(
        "Encrypting {} files with {}",
        compressed_files.len(),
        match encryption_method {
            EncryptionMethod::Password => "password",
            EncryptionMethod::Rsa { .. } => "RSA",
        }
    );

    // For password encryption, we need to prompt once and reuse
    // For RSA, each file can be encrypted independently
    match encryption_method {
        EncryptionMethod::Password => {
            // Prompt for password once
            let password = match prompt_password() {
                Ok(p) => p,
                Err(e) => {
                    let err_msg = format!("{}", e);
                    return compressed_files
                        .iter()
                        .map(|_| Err(JcError::Other(err_msg.clone())))
                        .collect();
                }
            };

            // Encrypt all files with the same password
            compressed_files
                .par_iter()
                .map(|file| {
                    encrypt_file_with_password(file, &password).map_err(|e| {
                        error!("Failed to encrypt {}: {}", file.display(), e);
                        e
                    })
                })
                .collect()
        }
        EncryptionMethod::Rsa { .. } => {
            // Each file can be encrypted independently
            compressed_files
                .par_iter()
                .map(|file| {
                    encrypt_file(file, encryption_method).map_err(|e| {
                        error!("Failed to encrypt {}: {}", file.display(), e);
                        e
                    })
                })
                .collect()
        }
    }
}

/// Helper function to encrypt with a pre-obtained password
fn encrypt_file_with_password(compressed_file: &Path, password: &str) -> JcResult<PathBuf> {
    let compressed_data = fs::read(compressed_file)?;

    // Generate salt and nonce
    let salt = PasswordEncryption::generate_salt()
        .map_err(|e| JcError::Other(format!("Failed to generate salt: {}", e)))?;
    let nonce = PasswordEncryption::generate_nonce()
        .map_err(|e| JcError::Other(format!("Failed to generate nonce: {}", e)))?;

    // Derive key from password
    let params = Argon2Params::default();
    let key = PasswordEncryption::derive_key(password, &salt, &params)
        .map_err(|e| JcError::Other(format!("Key derivation failed: {}", e)))?;

    // Encrypt data
    let encrypted = PasswordEncryption::encrypt(&compressed_data, &key, &nonce)
        .map_err(|e| JcError::Other(format!("Encryption failed: {}", e)))?;

    let metadata = EncryptionMetadata::Password {
        salt,
        nonce,
        argon2_params: params,
    };

    // Create encrypted container
    let container = EncryptedContainer::new(EncryptionType::Password, metadata, encrypted);

    // Generate output filename
    let output_path = compressed_file.with_extension(format!(
        "{}.jcze",
        compressed_file
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    ));

    // Write encrypted container
    container
        .write_to_file(&output_path)
        .map_err(|e| JcError::Other(format!("Failed to write encrypted file: {}", e)))?;

    // Remove original compressed file
    fs::remove_file(compressed_file)?;

    Ok(output_path)
}
