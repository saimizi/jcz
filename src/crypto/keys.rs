//! Key management utilities

use super::{CryptoError, CryptoResult};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::fs;
use std::path::Path;

/// Validate that a key file exists and is readable
pub fn validate_key_file(path: &Path) -> CryptoResult<()> {
    if !path.exists() {
        return Err(CryptoError::KeyFileNotFound(path.to_path_buf()));
    }

    if !path.is_file() {
        return Err(CryptoError::KeyFileNotReadable(path.to_path_buf()));
    }

    // Try to read the file to ensure it's readable
    fs::metadata(path).map_err(|_| CryptoError::KeyFileNotReadable(path.to_path_buf()))?;

    Ok(())
}

/// Read and parse RSA private key from PEM file
pub fn read_private_key_pem(path: &Path) -> CryptoResult<RsaPrivateKey> {
    // Validate file exists and is readable
    validate_key_file(path)?;

    // Read file contents
    let pem_data = fs::read_to_string(path)
        .map_err(|e| CryptoError::InvalidPemFormat(format!("Failed to read file: {}", e)))?;

    // Decode RSA private key from PEM
    let private_key = RsaPrivateKey::from_pkcs8_pem(&pem_data).map_err(|e| {
        CryptoError::InvalidPemFormat(format!("Failed to decode private key: {}", e))
    })?;

    // Validate key size
    let key_size = private_key.size() * 8; // size() returns bytes, convert to bits
    validate_key_size(key_size)?;

    Ok(private_key)
}

/// Read and parse RSA public key from PEM file
pub fn read_public_key_pem(path: &Path) -> CryptoResult<RsaPublicKey> {
    // Validate file exists and is readable
    validate_key_file(path)?;

    // Read file contents
    let pem_data = fs::read_to_string(path)
        .map_err(|e| CryptoError::InvalidPemFormat(format!("Failed to read file: {}", e)))?;

    // Decode RSA public key from PEM
    let public_key = RsaPublicKey::from_public_key_pem(&pem_data).map_err(|e| {
        CryptoError::InvalidPemFormat(format!("Failed to decode public key: {}", e))
    })?;

    // Validate key size
    let key_size = public_key.size() * 8; // size() returns bytes, convert to bits
    validate_key_size(key_size)?;

    Ok(public_key)
}

/// Validate RSA key size (minimum 2048 bits)
pub fn validate_key_size(key_bits: usize) -> CryptoResult<()> {
    const MIN_KEY_SIZE: usize = 2048;
    if key_bits < MIN_KEY_SIZE {
        return Err(CryptoError::KeySizeTooSmall {
            actual: key_bits,
            minimum: MIN_KEY_SIZE,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
    use rsa::RsaPrivateKey;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_key_pair() -> (RsaPrivateKey, RsaPublicKey) {
        use rand::rngs::OsRng;
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        (private_key, public_key)
    }

    #[test]
    fn test_validate_key_file_not_found() {
        let result = validate_key_file(Path::new("/nonexistent/file.pem"));
        assert!(result.is_err());
        if let Err(CryptoError::KeyFileNotFound(_)) = result {
            // Expected
        } else {
            panic!("Expected KeyFileNotFound error");
        }
    }

    #[test]
    fn test_read_private_key_pem() {
        let (private_key, _) = create_test_key_pair();

        // Write to temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        let pem_data = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .unwrap()
            .to_string();
        temp_file.write_all(pem_data.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Read back
        let loaded_key = read_private_key_pem(temp_file.path()).unwrap();

        // Verify key size
        assert_eq!(loaded_key.size(), private_key.size());
    }

    #[test]
    fn test_read_public_key_pem() {
        let (_, public_key) = create_test_key_pair();

        // Write to temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        let pem_data = public_key.to_public_key_pem(LineEnding::LF).unwrap();
        temp_file.write_all(pem_data.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Read back
        let loaded_key = read_public_key_pem(temp_file.path()).unwrap();

        // Verify key size
        assert_eq!(loaded_key.size(), public_key.size());
    }

    #[test]
    fn test_validate_key_size() {
        assert!(validate_key_size(2048).is_ok());
        assert!(validate_key_size(4096).is_ok());
        assert!(validate_key_size(1024).is_err());

        if let Err(CryptoError::KeySizeTooSmall { actual, minimum }) = validate_key_size(1024) {
            assert_eq!(actual, 1024);
            assert_eq!(minimum, 2048);
        } else {
            panic!("Expected KeySizeTooSmall error");
        }
    }

    #[test]
    fn test_read_invalid_pem() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"not a valid PEM file").unwrap();
        temp_file.flush().unwrap();

        let result = read_private_key_pem(temp_file.path());
        assert!(result.is_err());
        if let Err(CryptoError::InvalidPemFormat(_)) = result {
            // Expected
        } else {
            panic!("Expected InvalidPemFormat error");
        }
    }
}
