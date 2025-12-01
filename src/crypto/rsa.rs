//! RSA encryption implementation

use super::{CryptoError, CryptoResult};
use crate::crypto::keys::read_private_key_pem;
use ring::aead::{
    Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM,
};
use ring::error::Unspecified;
use ring::rand::{SecureRandom, SystemRandom};
use rsa::Oaep;
use sha2::Sha256;
use std::path::Path;

/// RSA encryption operations
pub struct RsaEncryption;

impl RsaEncryption {
    /// Generate random symmetric key for AES-256-GCM using cryptographically secure RNG
    pub fn generate_symmetric_key() -> CryptoResult<[u8; 32]> {
        let rng = SystemRandom::new();
        let mut key = [0u8; 32];
        rng.fill(&mut key).map_err(|_| {
            CryptoError::EncryptionFailed("Failed to generate symmetric key".to_string())
        })?;
        Ok(key)
    }

    /// Generate random nonce using cryptographically secure RNG
    pub fn generate_nonce() -> CryptoResult<[u8; 12]> {
        let rng = SystemRandom::new();
        let mut nonce = [0u8; 12];
        rng.fill(&mut nonce)
            .map_err(|_| CryptoError::EncryptionFailed("Failed to generate nonce".to_string()))?;
        Ok(nonce)
    }

    /// Encrypt symmetric key with RSA public key using OAEP padding
    /// Note: Despite the requirements saying "private key", standard RSA encryption
    /// uses the public key to encrypt (so only the private key holder can decrypt).
    /// The CLI will accept a public key path for encryption.
    pub fn encrypt_symmetric_key(
        symmetric_key: &[u8; 32],
        public_key_path: &Path,
    ) -> CryptoResult<Vec<u8>> {
        // Read and parse public key
        use crate::crypto::keys::read_public_key_pem;
        let public_key = read_public_key_pem(public_key_path)?;

        // Use OAEP padding with SHA-256
        let padding = Oaep::new::<Sha256>();

        // Encrypt the symmetric key with the public key
        use rand::rngs::OsRng;
        let mut rng = OsRng;
        let encrypted_key = public_key
            .encrypt(&mut rng, padding, symmetric_key)
            .map_err(|e| {
                CryptoError::RsaError(format!("Failed to encrypt symmetric key: {}", e))
            })?;

        Ok(encrypted_key)
    }

    /// Decrypt symmetric key with RSA private key using OAEP padding
    /// Note: Despite the requirements saying "public key", standard RSA decryption
    /// uses the private key to decrypt (after encryption with the public key).
    /// The CLI will accept a private key path for decryption.
    pub fn decrypt_symmetric_key(
        encrypted_key: &[u8],
        private_key_path: &Path,
    ) -> CryptoResult<[u8; 32]> {
        // Read and parse private key
        let private_key = read_private_key_pem(private_key_path)?;

        // Use OAEP padding with SHA-256
        let padding = Oaep::new::<Sha256>();

        // Decrypt the symmetric key with the private key
        let decrypted = private_key.decrypt(padding, encrypted_key).map_err(|e| {
            CryptoError::RsaError(format!("Failed to decrypt symmetric key: {}", e))
        })?;

        // Ensure we got exactly 32 bytes
        if decrypted.len() != 32 {
            return Err(CryptoError::DecryptionFailed(format!(
                "Expected 32 bytes, got {}",
                decrypted.len()
            )));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&decrypted);
        Ok(key)
    }

    /// Encrypt data with AES-256-GCM using symmetric key
    pub fn encrypt_data(data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> CryptoResult<Vec<u8>> {
        // Create a nonce sequence that returns our nonce once
        struct SingleNonce([u8; 12]);

        impl NonceSequence for SingleNonce {
            fn advance(&mut self) -> Result<Nonce, Unspecified> {
                Nonce::try_assume_unique_for_key(&self.0)
            }
        }

        // Create sealing key
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|_| CryptoError::EncryptionFailed("Failed to create key".to_string()))?;
        let nonce_sequence = SingleNonce(*nonce);
        let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

        // Prepare data for encryption (ring modifies in place)
        let mut in_out = data.to_vec();

        // Seal (encrypt and authenticate)
        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut in_out)
            .map_err(|_| CryptoError::EncryptionFailed("Encryption failed".to_string()))?;

        Ok(in_out)
    }

    /// Decrypt data with AES-256-GCM using symmetric key
    pub fn decrypt_data(
        encrypted_data: &[u8],
        key: &[u8; 32],
        nonce: &[u8; 12],
    ) -> CryptoResult<Vec<u8>> {
        // Create a nonce sequence that returns our nonce once
        struct SingleNonce([u8; 12]);

        impl NonceSequence for SingleNonce {
            fn advance(&mut self) -> Result<Nonce, Unspecified> {
                Nonce::try_assume_unique_for_key(&self.0)
            }
        }

        // Create opening key
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|_| CryptoError::DecryptionFailed("Failed to create key".to_string()))?;
        let nonce_sequence = SingleNonce(*nonce);
        let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);

        // Prepare data for decryption (ring modifies in place)
        let mut in_out = encrypted_data.to_vec();

        // Open (decrypt and verify authentication)
        let decrypted = opening_key
            .open_in_place(Aad::empty(), &mut in_out)
            .map_err(|_| CryptoError::AuthenticationFailed)?;

        Ok(decrypted.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
    use rsa::RsaPrivateKey;
    use std::io::Write;
    use tempfile::NamedTempFile;

    pub(super) fn create_test_key_pair() -> (NamedTempFile, NamedTempFile) {
        use rand::rngs::OsRng;
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = rsa::RsaPublicKey::from(&private_key);

        // Write private key to temp file
        let mut priv_file = NamedTempFile::new().unwrap();
        let priv_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .unwrap()
            .to_string();
        priv_file.write_all(priv_pem.as_bytes()).unwrap();
        priv_file.flush().unwrap();

        // Write public key to temp file
        let mut pub_file = NamedTempFile::new().unwrap();
        let pub_pem = public_key.to_public_key_pem(LineEnding::LF).unwrap();
        pub_file.write_all(pub_pem.as_bytes()).unwrap();
        pub_file.flush().unwrap();

        (priv_file, pub_file)
    }

    #[test]
    fn test_generate_symmetric_key() {
        let key1 = RsaEncryption::generate_symmetric_key().unwrap();
        let key2 = RsaEncryption::generate_symmetric_key().unwrap();

        // Keys should be different
        assert_ne!(key1, key2);
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = RsaEncryption::generate_nonce().unwrap();
        let nonce2 = RsaEncryption::generate_nonce().unwrap();

        // Nonces should be different
        assert_ne!(nonce1, nonce2);
        assert_eq!(nonce1.len(), 12);
        assert_eq!(nonce2.len(), 12);
    }

    #[test]
    fn test_rsa_symmetric_key_round_trip() {
        let (priv_file, pub_file) = create_test_key_pair();
        let symmetric_key = RsaEncryption::generate_symmetric_key().unwrap();

        // Encrypt with public key
        let encrypted =
            RsaEncryption::encrypt_symmetric_key(&symmetric_key, pub_file.path()).unwrap();

        // Decrypt with private key
        let decrypted = RsaEncryption::decrypt_symmetric_key(&encrypted, priv_file.path()).unwrap();

        // Should match original
        assert_eq!(decrypted, symmetric_key);
    }

    #[test]
    fn test_encrypt_decrypt_data() {
        let data = b"Hello, RSA World!";
        let key = RsaEncryption::generate_symmetric_key().unwrap();
        let nonce = RsaEncryption::generate_nonce().unwrap();

        // Encrypt
        let encrypted = RsaEncryption::encrypt_data(data, &key, &nonce).unwrap();

        // Decrypt
        let decrypted = RsaEncryption::decrypt_data(&encrypted, &key, &nonce).unwrap();

        // Should match original
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_full_rsa_encryption_flow() {
        let (priv_file, pub_file) = create_test_key_pair();
        let data = b"Secret message for RSA encryption test";

        // Generate symmetric key and nonce
        let symmetric_key = RsaEncryption::generate_symmetric_key().unwrap();
        let nonce = RsaEncryption::generate_nonce().unwrap();

        // Encrypt data with symmetric key
        let encrypted_data = RsaEncryption::encrypt_data(data, &symmetric_key, &nonce).unwrap();

        // Encrypt symmetric key with RSA public key
        let encrypted_key =
            RsaEncryption::encrypt_symmetric_key(&symmetric_key, pub_file.path()).unwrap();

        // --- Decryption flow ---

        // Decrypt symmetric key with RSA private key
        let recovered_key =
            RsaEncryption::decrypt_symmetric_key(&encrypted_key, priv_file.path()).unwrap();

        // Decrypt data with recovered symmetric key
        let decrypted_data =
            RsaEncryption::decrypt_data(&encrypted_data, &recovered_key, &nonce).unwrap();

        // Should match original
        assert_eq!(decrypted_data, data);
    }
}

// Feature: file-encryption, Property 2: RSA encryption round-trip
// For any compressed file and any valid RSA key pair (public key for encryption,
// private key for decryption), encrypting the file with the public key and then
// decrypting with the corresponding private key should produce data that decompresses
// to the original content.

#[cfg(test)]
mod proptests {
    use super::tests::create_test_key_pair;
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))] // Reduced from 100 due to RSA key generation cost

        #[test]
        fn prop_rsa_encryption_round_trip(
            data in prop::collection::vec(any::<u8>(), 0..5000),
        ) {
            let (priv_file, pub_file) = create_test_key_pair();

            // Generate symmetric key and nonce
            let symmetric_key = RsaEncryption::generate_symmetric_key().unwrap();
            let nonce = RsaEncryption::generate_nonce().unwrap();

            // Encrypt data with symmetric key
            let encrypted_data = RsaEncryption::encrypt_data(&data, &symmetric_key, &nonce).unwrap();

            // Encrypt symmetric key with RSA public key
            let encrypted_key = RsaEncryption::encrypt_symmetric_key(&symmetric_key, pub_file.path()).unwrap();

            // Decrypt symmetric key with RSA private key
            let recovered_key = RsaEncryption::decrypt_symmetric_key(&encrypted_key, priv_file.path()).unwrap();

            // Decrypt data with recovered symmetric key
            let decrypted_data = RsaEncryption::decrypt_data(&encrypted_data, &recovered_key, &nonce).unwrap();

            // Should match original
            assert_eq!(decrypted_data, data);
        }

        #[test]
        fn prop_different_symmetric_keys_produce_different_ciphertext(
            data in prop::collection::vec(any::<u8>(), 100..1000),
        ) {
            let nonce = RsaEncryption::generate_nonce().unwrap();
            let key1 = RsaEncryption::generate_symmetric_key().unwrap();
            let key2 = RsaEncryption::generate_symmetric_key().unwrap();

            let encrypted1 = RsaEncryption::encrypt_data(&data, &key1, &nonce).unwrap();
            let encrypted2 = RsaEncryption::encrypt_data(&data, &key2, &nonce).unwrap();

            // Different keys should produce different ciphertext
            assert_ne!(encrypted1, encrypted2);
        }
    }
}
