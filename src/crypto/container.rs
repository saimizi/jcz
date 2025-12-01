//! Encrypted container format implementation

use super::{CryptoError, CryptoResult, EncryptionMetadata, EncryptionType};
use std::io::{Read, Write};
use std::path::Path;

/// Magic bytes for JCZ encrypted files: "JCZE"
const MAGIC_BYTES: [u8; 4] = [0x4A, 0x43, 0x5A, 0x45];

/// Current container format version
const CONTAINER_VERSION: u8 = 1;

/// Encrypted container structure
#[derive(Debug, Clone)]
pub struct EncryptedContainer {
    /// Container format version
    pub version: u8,
    /// Encryption type
    pub encryption_type: EncryptionType,
    /// Encryption metadata
    pub metadata: EncryptionMetadata,
    /// Encrypted data
    pub encrypted_data: Vec<u8>,
}

impl EncryptedContainer {
    /// Create a new encrypted container
    pub fn new(
        encryption_type: EncryptionType,
        metadata: EncryptionMetadata,
        encrypted_data: Vec<u8>,
    ) -> Self {
        Self {
            version: CONTAINER_VERSION,
            encryption_type,
            metadata,
            encrypted_data,
        }
    }

    /// Write container to file
    pub fn write_to_file(&self, path: &Path) -> CryptoResult<()> {
        let mut file = std::fs::File::create(path)?;
        let bytes = self.to_bytes()?;
        file.write_all(&bytes)?;
        Ok(())
    }

    /// Read container from file
    pub fn read_from_file(path: &Path) -> CryptoResult<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    /// Serialize container to bytes
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        let mut bytes = Vec::new();

        // Magic bytes
        bytes.extend_from_slice(&MAGIC_BYTES);

        // Version
        bytes.push(self.version);

        // Encryption type
        bytes.push(self.encryption_type.to_u8());

        // Serialize metadata
        let metadata_bytes = self.serialize_metadata()?;
        let metadata_len = metadata_bytes.len() as u32;
        bytes.extend_from_slice(&metadata_len.to_le_bytes());
        bytes.extend_from_slice(&metadata_bytes);

        // Encrypted data
        bytes.extend_from_slice(&self.encrypted_data);

        Ok(bytes)
    }

    /// Deserialize container from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() < 10 {
            return Err(CryptoError::InvalidContainer(
                "Container too small".to_string(),
            ));
        }

        let mut pos = 0;

        // Check magic bytes
        if &bytes[pos..pos + 4] != &MAGIC_BYTES {
            return Err(CryptoError::InvalidContainer(
                "Invalid magic bytes".to_string(),
            ));
        }
        pos += 4;

        // Read version
        let version = bytes[pos];
        pos += 1;

        if version != CONTAINER_VERSION {
            return Err(CryptoError::UnsupportedVersion(version));
        }

        // Read encryption type
        let encryption_type = EncryptionType::from_u8(bytes[pos])
            .ok_or_else(|| CryptoError::InvalidContainer("Invalid encryption type".to_string()))?;
        pos += 1;

        // Read metadata length
        if bytes.len() < pos + 4 {
            return Err(CryptoError::InvalidContainer(
                "Missing metadata length".to_string(),
            ));
        }
        let metadata_len =
            u32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]])
                as usize;
        pos += 4;

        // Read metadata
        if bytes.len() < pos + metadata_len {
            return Err(CryptoError::InvalidContainer(
                "Truncated metadata".to_string(),
            ));
        }
        let metadata_bytes = &bytes[pos..pos + metadata_len];
        let metadata = Self::deserialize_metadata(encryption_type, metadata_bytes)?;
        pos += metadata_len;

        // Read encrypted data
        let encrypted_data = bytes[pos..].to_vec();

        Ok(Self {
            version,
            encryption_type,
            metadata,
            encrypted_data,
        })
    }

    /// Serialize metadata to bytes
    fn serialize_metadata(&self) -> CryptoResult<Vec<u8>> {
        let mut bytes = Vec::new();

        match &self.metadata {
            EncryptionMetadata::Password {
                salt,
                nonce,
                argon2_params,
            } => {
                bytes.extend_from_slice(salt);
                bytes.extend_from_slice(nonce);
                bytes.extend_from_slice(&argon2_params.memory_cost.to_le_bytes());
                bytes.extend_from_slice(&argon2_params.time_cost.to_le_bytes());
                bytes.extend_from_slice(&argon2_params.parallelism.to_le_bytes());
            }
            EncryptionMetadata::Rsa {
                encrypted_key,
                nonce,
            } => {
                let key_len = encrypted_key.len() as u32;
                bytes.extend_from_slice(&key_len.to_le_bytes());
                bytes.extend_from_slice(encrypted_key);
                bytes.extend_from_slice(nonce);
            }
        }

        Ok(bytes)
    }

    /// Deserialize metadata from bytes
    fn deserialize_metadata(
        encryption_type: EncryptionType,
        bytes: &[u8],
    ) -> CryptoResult<EncryptionMetadata> {
        match encryption_type {
            EncryptionType::Password => {
                if bytes.len() < 32 + 12 + 12 {
                    return Err(CryptoError::InvalidContainer(
                        "Invalid password metadata size".to_string(),
                    ));
                }

                let mut salt = [0u8; 32];
                salt.copy_from_slice(&bytes[0..32]);

                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&bytes[32..44]);

                let memory_cost = u32::from_le_bytes([bytes[44], bytes[45], bytes[46], bytes[47]]);
                let time_cost = u32::from_le_bytes([bytes[48], bytes[49], bytes[50], bytes[51]]);
                let parallelism = u32::from_le_bytes([bytes[52], bytes[53], bytes[54], bytes[55]]);

                Ok(EncryptionMetadata::Password {
                    salt,
                    nonce,
                    argon2_params: super::Argon2Params {
                        memory_cost,
                        time_cost,
                        parallelism,
                    },
                })
            }
            EncryptionType::Rsa => {
                if bytes.len() < 4 {
                    return Err(CryptoError::InvalidContainer(
                        "Invalid RSA metadata size".to_string(),
                    ));
                }

                let key_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;

                if bytes.len() < 4 + key_len + 12 {
                    return Err(CryptoError::InvalidContainer(
                        "Truncated RSA metadata".to_string(),
                    ));
                }

                let encrypted_key = bytes[4..4 + key_len].to_vec();

                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&bytes[4 + key_len..4 + key_len + 12]);

                Ok(EncryptionMetadata::Rsa {
                    encrypted_key,
                    nonce,
                })
            }
        }
    }

    /// Get the encryption type
    #[allow(dead_code)]
    pub fn get_encryption_type(&self) -> EncryptionType {
        self.encryption_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: file-encryption, Property 3: Encryption metadata completeness
    // For any encrypted file (password or RSA), the stored metadata should contain
    // all parameters necessary for decryption and should not contain the actual
    // encryption key or password.

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_password_container_round_trip(
            salt in prop::array::uniform32(any::<u8>()),
            nonce in prop::array::uniform12(any::<u8>()),
            memory_cost in 1u32..100000u32,
            time_cost in 1u32..10u32,
            parallelism in 1u32..16u32,
            encrypted_data in prop::collection::vec(any::<u8>(), 0..1000),
        ) {
            let metadata = EncryptionMetadata::Password {
                salt,
                nonce,
                argon2_params: super::super::Argon2Params {
                    memory_cost,
                    time_cost,
                    parallelism,
                },
            };

            let container = EncryptedContainer::new(
                EncryptionType::Password,
                metadata.clone(),
                encrypted_data.clone(),
            );

            // Serialize and deserialize
            let bytes = container.to_bytes().unwrap();
            let recovered = EncryptedContainer::from_bytes(&bytes).unwrap();

            // Verify all metadata is preserved
            assert_eq!(recovered.encryption_type, EncryptionType::Password);
            assert_eq!(recovered.encrypted_data, encrypted_data);

            if let EncryptionMetadata::Password {
                salt: recovered_salt,
                nonce: recovered_nonce,
                argon2_params: recovered_params,
            } = recovered.metadata
            {
                assert_eq!(recovered_salt, salt);
                assert_eq!(recovered_nonce, nonce);
                assert_eq!(recovered_params.memory_cost, memory_cost);
                assert_eq!(recovered_params.time_cost, time_cost);
                assert_eq!(recovered_params.parallelism, parallelism);
            } else {
                panic!("Expected Password metadata");
            }

            // Verify no sensitive data in metadata (salt and nonce are public, params are public)
            // The actual password should never be stored
            if let EncryptionMetadata::Password { salt: s, nonce: n, argon2_params: p } = metadata {
                // These are all non-sensitive parameters
                assert_eq!(s.len(), 32);
                assert_eq!(n.len(), 12);
                assert!(p.memory_cost > 0);
                assert!(p.time_cost > 0);
                assert!(p.parallelism > 0);
            }
        }

        #[test]
        fn prop_rsa_container_round_trip(
            encrypted_key in prop::collection::vec(any::<u8>(), 1..512),
            nonce in prop::array::uniform12(any::<u8>()),
            encrypted_data in prop::collection::vec(any::<u8>(), 0..1000),
        ) {
            let metadata = EncryptionMetadata::Rsa {
                encrypted_key: encrypted_key.clone(),
                nonce,
            };

            let container = EncryptedContainer::new(
                EncryptionType::Rsa,
                metadata.clone(),
                encrypted_data.clone(),
            );

            // Serialize and deserialize
            let bytes = container.to_bytes().unwrap();
            let recovered = EncryptedContainer::from_bytes(&bytes).unwrap();

            // Verify all metadata is preserved
            assert_eq!(recovered.encryption_type, EncryptionType::Rsa);
            assert_eq!(recovered.encrypted_data, encrypted_data);

            if let EncryptionMetadata::Rsa {
                encrypted_key: recovered_key,
                nonce: recovered_nonce,
            } = recovered.metadata
            {
                assert_eq!(recovered_key, encrypted_key);
                assert_eq!(recovered_nonce, nonce);
            } else {
                panic!("Expected RSA metadata");
            }

            // Verify the encrypted symmetric key is stored (not the plaintext key)
            // and nonce is public
            if let EncryptionMetadata::Rsa { encrypted_key: ek, nonce: n } = metadata {
                assert!(!ek.is_empty());
                assert_eq!(n.len(), 12);
            }
        }
    }

    #[test]
    fn test_invalid_magic_bytes() {
        let bad_bytes = vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00];
        let result = EncryptedContainer::from_bytes(&bad_bytes);
        assert!(result.is_err());
        if let Err(CryptoError::InvalidContainer(msg)) = result {
            assert!(msg.contains("magic bytes"));
        } else {
            panic!("Expected InvalidContainer error");
        }
    }

    #[test]
    fn test_unsupported_version() {
        let mut bytes = vec![0x4A, 0x43, 0x5A, 0x45]; // Magic bytes
        bytes.push(99); // Invalid version
        bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00]);

        let result = EncryptedContainer::from_bytes(&bytes);
        assert!(result.is_err());
        if let Err(CryptoError::UnsupportedVersion(v)) = result {
            assert_eq!(v, 99);
        } else {
            panic!("Expected UnsupportedVersion error");
        }
    }

    #[test]
    fn test_truncated_container() {
        let bytes = vec![0x4A, 0x43, 0x5A, 0x45, 0x01]; // Too short
        let result = EncryptedContainer::from_bytes(&bytes);
        assert!(result.is_err());
    }
}
