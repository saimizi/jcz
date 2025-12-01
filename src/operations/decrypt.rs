//! Decryption operations for encrypted files

use crate::core::config::DecryptionMethod;
use crate::core::error::{JcError, JcResult};
use crate::crypto::{EncryptedContainer, EncryptionMetadata, PasswordEncryption, RsaEncryption};
use crate::utils::{error, info};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Prompt user for password securely (without echo)
fn prompt_password() -> JcResult<String> {
    use std::io::{self, Write};

    print!("Enter decryption password: ");
    io::stdout().flush()?;

    let password = rpassword::read_password()
        .map_err(|e| JcError::Other(format!("Failed to read password: {}", e)))?;

    if password.is_empty() {
        return Err(JcError::Other("Password cannot be empty".to_string()));
    }

    Ok(password)
}

/// Check if a file is encrypted by looking for .jcze extension
pub fn is_encrypted_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "jcze")
        .unwrap_or(false)
}

/// Decrypt a single encrypted file
pub fn decrypt_file(
    encrypted_file: &Path,
    decryption_method: Option<&DecryptionMethod>,
    remove_encrypted: bool,
) -> JcResult<PathBuf> {
    // Check if file is encrypted
    if !is_encrypted_file(encrypted_file) {
        // Not encrypted, return as-is
        return Ok(encrypted_file.to_path_buf());
    }

    info!("Decrypting file: {}", encrypted_file.display());

    // Read encrypted container
    let container = EncryptedContainer::read_from_file(encrypted_file)
        .map_err(|e| JcError::Other(format!("Failed to read encrypted file: {}", e)))?;

    // Decrypt based on container type and provided method
    let decrypted_data = match (&container.metadata, decryption_method) {
        (
            EncryptionMetadata::Password {
                salt,
                nonce,
                argon2_params,
            },
            _,
        ) => {
            // Password encryption - prompt for password
            let password = prompt_password()?;

            // Derive key
            let key = PasswordEncryption::derive_key(&password, salt, argon2_params)
                .map_err(|e| JcError::Other(format!("Key derivation failed: {}", e)))?;

            // Decrypt
            PasswordEncryption::decrypt(&container.encrypted_data, &key, nonce)
                .map_err(|e| JcError::Other(format!("Decryption failed: {}", e)))?
        }
        (
            EncryptionMetadata::Rsa {
                encrypted_key,
                nonce,
            },
            Some(DecryptionMethod::Rsa { private_key_path }),
        ) => {
            // RSA encryption with provided private key
            let symmetric_key =
                RsaEncryption::decrypt_symmetric_key(encrypted_key, private_key_path).map_err(
                    |e| JcError::Other(format!("Failed to decrypt symmetric key: {}", e)),
                )?;

            RsaEncryption::decrypt_data(&container.encrypted_data, &symmetric_key, nonce)
                .map_err(|e| JcError::Other(format!("Decryption failed: {}", e)))?
        }
        (EncryptionMetadata::Rsa { .. }, _) => {
            return Err(JcError::Other(
                "RSA encrypted file requires --decrypt-key option".to_string(),
            ));
        }
    };

    // Generate output filename by removing .jcze extension
    let output_path = encrypted_file.with_extension("");

    // Write decrypted data
    fs::write(&output_path, &decrypted_data)?;

    info!("Decrypted file created: {}", output_path.display());

    // Remove encrypted file only if requested
    if remove_encrypted {
        fs::remove_file(encrypted_file)?;
        info!("Removed encrypted file: {}", encrypted_file.display());
    }

    Ok(output_path)
}

/// Decrypt multiple encrypted files in parallel
#[allow(dead_code)]
pub fn decrypt_files(
    encrypted_files: Vec<PathBuf>,
    decryption_method: Option<&DecryptionMethod>,
) -> Vec<JcResult<PathBuf>> {
    info!("Decrypting {} files", encrypted_files.len());

    // Check if any files are password-encrypted
    let has_password_encrypted = encrypted_files.iter().any(|f| {
        if let Ok(container) = EncryptedContainer::read_from_file(f) {
            matches!(container.metadata, EncryptionMetadata::Password { .. })
        } else {
            false
        }
    });

    if has_password_encrypted {
        // Prompt for password once
        let password = match prompt_password() {
            Ok(p) => p,
            Err(e) => {
                let err_msg = format!("{}", e);
                return encrypted_files
                    .iter()
                    .map(|_| Err(JcError::Other(err_msg.clone())))
                    .collect();
            }
        };

        // Decrypt all files
        encrypted_files
            .par_iter()
            .map(|file| {
                decrypt_file_with_password(file, &password, decryption_method, false).map_err(|e| {
                    error!("Failed to decrypt {}: {}", file.display(), e);
                    e
                })
            })
            .collect()
    } else {
        // No password encryption, decrypt independently
        encrypted_files
            .par_iter()
            .map(|file| {
                decrypt_file(file, decryption_method, false).map_err(|e| {
                    error!("Failed to decrypt {}: {}", file.display(), e);
                    e
                })
            })
            .collect()
    }
}

/// Helper function to decrypt with a pre-obtained password
#[allow(dead_code)]
fn decrypt_file_with_password(
    encrypted_file: &Path,
    password: &str,
    decryption_method: Option<&DecryptionMethod>,
    remove_encrypted: bool,
) -> JcResult<PathBuf> {
    if !is_encrypted_file(encrypted_file) {
        return Ok(encrypted_file.to_path_buf());
    }

    let container = EncryptedContainer::read_from_file(encrypted_file)
        .map_err(|e| JcError::Other(format!("Failed to read encrypted file: {}", e)))?;

    let decrypted_data = match (&container.metadata, decryption_method) {
        (
            EncryptionMetadata::Password {
                salt,
                nonce,
                argon2_params,
            },
            _,
        ) => {
            let key = PasswordEncryption::derive_key(password, salt, argon2_params)
                .map_err(|e| JcError::Other(format!("Key derivation failed: {}", e)))?;

            PasswordEncryption::decrypt(&container.encrypted_data, &key, nonce)
                .map_err(|e| JcError::Other(format!("Decryption failed: {}", e)))?
        }
        (
            EncryptionMetadata::Rsa {
                encrypted_key,
                nonce,
            },
            Some(DecryptionMethod::Rsa { private_key_path }),
        ) => {
            let symmetric_key =
                RsaEncryption::decrypt_symmetric_key(encrypted_key, private_key_path).map_err(
                    |e| JcError::Other(format!("Failed to decrypt symmetric key: {}", e)),
                )?;

            RsaEncryption::decrypt_data(&container.encrypted_data, &symmetric_key, nonce)
                .map_err(|e| JcError::Other(format!("Decryption failed: {}", e)))?
        }
        (EncryptionMetadata::Rsa { .. }, _) => {
            return Err(JcError::Other(
                "RSA encrypted file requires --decrypt-key option".to_string(),
            ));
        }
    };

    let output_path = encrypted_file.with_extension("");
    fs::write(&output_path, &decrypted_data)?;

    // Remove encrypted file only if requested
    if remove_encrypted {
        fs::remove_file(encrypted_file)?;
    }

    Ok(output_path)
}
