//! Password-based encryption implementation

use super::{Argon2Params, CryptoError, CryptoResult};
use argon2::{Argon2, Version};
use ring::rand::{SecureRandom, SystemRandom};

/// Password-based encryption operations
pub struct PasswordEncryption;

impl PasswordEncryption {
    /// Validate password (reject empty passwords)
    pub fn validate_password(password: &str) -> CryptoResult<()> {
        if password.is_empty() {
            return Err(CryptoError::InvalidPassword);
        }
        Ok(())
    }

    /// Derive encryption key from password using Argon2id
    pub fn derive_key(
        password: &str,
        salt: &[u8; 32],
        params: &Argon2Params,
    ) -> CryptoResult<[u8; 32]> {
        // Validate password
        Self::validate_password(password)?;

        // Build Argon2 parameters
        let argon2_params = argon2::Params::new(
            params.memory_cost,
            params.time_cost,
            params.parallelism,
            Some(32),
        )
        .map_err(|e| CryptoError::KeyDerivationFailed(format!("Invalid params: {}", e)))?;

        // Create Argon2id instance
        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, argon2_params);

        // Derive key directly into output buffer
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| {
                CryptoError::KeyDerivationFailed(format!("Key derivation failed: {}", e))
            })?;

        Ok(key)
    }

    /// Generate random salt using cryptographically secure RNG
    pub fn generate_salt() -> CryptoResult<[u8; 32]> {
        let rng = SystemRandom::new();
        let mut salt = [0u8; 32];
        rng.fill(&mut salt)
            .map_err(|_| CryptoError::KeyDerivationFailed("Failed to generate salt".to_string()))?;
        Ok(salt)
    }

    /// Generate random nonce using cryptographically secure RNG
    pub fn generate_nonce() -> CryptoResult<[u8; 12]> {
        let rng = SystemRandom::new();
        let mut nonce = [0u8; 12];
        rng.fill(&mut nonce)
            .map_err(|_| CryptoError::EncryptionFailed("Failed to generate nonce".to_string()))?;
        Ok(nonce)
    }

    /// Encrypt data with AES-256-GCM
    pub fn encrypt(data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> CryptoResult<Vec<u8>> {
        use ring::aead::{
            Aad, BoundKey, Nonce, NonceSequence, SealingKey, UnboundKey, AES_256_GCM,
        };
        use ring::error::Unspecified;

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

    /// Decrypt data with AES-256-GCM
    pub fn decrypt(
        encrypted_data: &[u8],
        key: &[u8; 32],
        nonce: &[u8; 12],
    ) -> CryptoResult<Vec<u8>> {
        use ring::aead::{
            Aad, BoundKey, Nonce, NonceSequence, OpeningKey, UnboundKey, AES_256_GCM,
        };
        use ring::error::Unspecified;

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

    #[test]
    fn test_validate_password() {
        assert!(PasswordEncryption::validate_password("valid_password").is_ok());
        assert!(PasswordEncryption::validate_password("").is_err());
    }

    #[test]
    fn test_generate_salt() {
        let salt1 = PasswordEncryption::generate_salt().unwrap();
        let salt2 = PasswordEncryption::generate_salt().unwrap();

        // Salts should be different
        assert_ne!(salt1, salt2);
        assert_eq!(salt1.len(), 32);
        assert_eq!(salt2.len(), 32);
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = PasswordEncryption::generate_nonce().unwrap();
        let nonce2 = PasswordEncryption::generate_nonce().unwrap();

        // Nonces should be different
        assert_ne!(nonce1, nonce2);
        assert_eq!(nonce1.len(), 12);
        assert_eq!(nonce2.len(), 12);
    }

    #[test]
    fn test_encrypt() {
        let data = b"Hello, World!";
        let key = [42u8; 32];
        let nonce = [1u8; 12];

        let encrypted = PasswordEncryption::encrypt(data, &key, &nonce).unwrap();

        // Encrypted data should be longer (includes auth tag)
        assert!(encrypted.len() > data.len());
        // Encrypted data should be different from plaintext
        assert_ne!(&encrypted[..data.len()], data);
    }

    #[test]
    fn test_decrypt() {
        let data = b"Hello, World!";
        let key = [42u8; 32];
        let nonce = [1u8; 12];

        // First encrypt
        let encrypted = PasswordEncryption::encrypt(data, &key, &nonce).unwrap();

        // Then decrypt
        let decrypted = PasswordEncryption::decrypt(&encrypted, &key, &nonce).unwrap();

        // Should match original
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let data = b"Hello, World!";
        let key1 = [42u8; 32];
        let key2 = [43u8; 32];
        let nonce = [1u8; 12];

        let encrypted = PasswordEncryption::encrypt(data, &key1, &nonce).unwrap();

        // Try to decrypt with wrong key
        let result = PasswordEncryption::decrypt(&encrypted, &key2, &nonce);
        assert!(result.is_err());
        if let Err(CryptoError::AuthenticationFailed) = result {
            // Expected
        } else {
            panic!("Expected AuthenticationFailed error");
        }
    }

    #[test]
    fn test_decrypt_wrong_nonce() {
        let data = b"Hello, World!";
        let key = [42u8; 32];
        let nonce1 = [1u8; 12];
        let nonce2 = [2u8; 12];

        let encrypted = PasswordEncryption::encrypt(data, &key, &nonce1).unwrap();

        // Try to decrypt with wrong nonce
        let result = PasswordEncryption::decrypt(&encrypted, &key, &nonce2);
        assert!(result.is_err());
        if let Err(CryptoError::AuthenticationFailed) = result {
            // Expected
        } else {
            panic!("Expected AuthenticationFailed error");
        }
    }

    #[test]
    fn test_decrypt_corrupted_data() {
        let data = b"Hello, World!";
        let key = [42u8; 32];
        let nonce = [1u8; 12];

        let mut encrypted = PasswordEncryption::encrypt(data, &key, &nonce).unwrap();

        // Corrupt the data
        encrypted[0] ^= 0xFF;

        // Try to decrypt corrupted data
        let result = PasswordEncryption::decrypt(&encrypted, &key, &nonce);
        assert!(result.is_err());
        if let Err(CryptoError::AuthenticationFailed) = result {
            // Expected
        } else {
            panic!("Expected AuthenticationFailed error");
        }
    }

    #[test]
    fn test_derive_key_deterministic() {
        let password = "test_password";
        let salt = [42u8; 32];
        let params = Argon2Params::default();

        let key1 = PasswordEncryption::derive_key(password, &salt, &params).unwrap();
        let key2 = PasswordEncryption::derive_key(password, &salt, &params).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let salt = [42u8; 32];
        let params = Argon2Params::default();

        let key1 = PasswordEncryption::derive_key("password1", &salt, &params).unwrap();
        let key2 = PasswordEncryption::derive_key("password2", &salt, &params).unwrap();

        // Different passwords should produce different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_key_different_salts() {
        let password = "test_password";
        let salt1 = [42u8; 32];
        let salt2 = [43u8; 32];
        let params = Argon2Params::default();

        let key1 = PasswordEncryption::derive_key(password, &salt1, &params).unwrap();
        let key2 = PasswordEncryption::derive_key(password, &salt2, &params).unwrap();

        // Different salts should produce different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_key_empty_password() {
        let salt = [42u8; 32];
        let params = Argon2Params::default();

        let result = PasswordEncryption::derive_key("", &salt, &params);
        assert!(result.is_err());
        if let Err(CryptoError::InvalidPassword) = result {
            // Expected
        } else {
            panic!("Expected InvalidPassword error");
        }
    }
}

// Feature: file-encryption, Property 1: Password encryption round-trip
// For any compressed file and any non-empty password, encrypting the file with that password
// and then decrypting with the same password should produce data that decompresses to the
// original content.

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Use lighter Argon2 parameters for testing to speed up tests
    fn test_params() -> Argon2Params {
        Argon2Params {
            memory_cost: 1024, // 1MB instead of 64MB
            time_cost: 1,      // 1 iteration instead of 3
            parallelism: 1,    // 1 thread instead of 4
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_password_encryption_round_trip(
            data in prop::collection::vec(any::<u8>(), 0..10000),
            password in "[a-zA-Z0-9]{1,100}",
        ) {
            let salt = PasswordEncryption::generate_salt().unwrap();
            let nonce = PasswordEncryption::generate_nonce().unwrap();
            let params = test_params();

            // Derive key from password
            let key = PasswordEncryption::derive_key(&password, &salt, &params).unwrap();

            // Encrypt
            let encrypted = PasswordEncryption::encrypt(&data, &key, &nonce).unwrap();

            // Decrypt with same password
            let key2 = PasswordEncryption::derive_key(&password, &salt, &params).unwrap();
            let decrypted = PasswordEncryption::decrypt(&encrypted, &key2, &nonce).unwrap();

            // Should match original
            assert_eq!(decrypted, data);
        }

        #[test]
        fn prop_different_passwords_produce_different_keys(
            password1 in "[a-zA-Z0-9]{1,100}",
            password2 in "[a-zA-Z0-9]{1,100}",
        ) {
            prop_assume!(password1 != password2);

            let salt = [42u8; 32];
            let params = test_params();

            let key1 = PasswordEncryption::derive_key(&password1, &salt, &params).unwrap();
            let key2 = PasswordEncryption::derive_key(&password2, &salt, &params).unwrap();

            assert_ne!(key1, key2);
        }

        #[test]
        fn prop_same_password_same_salt_produces_same_key(
            password in "[a-zA-Z0-9]{1,100}",
            salt in prop::array::uniform32(any::<u8>()),
        ) {
            let params = test_params();

            let key1 = PasswordEncryption::derive_key(&password, &salt, &params).unwrap();
            let key2 = PasswordEncryption::derive_key(&password, &salt, &params).unwrap();

            assert_eq!(key1, key2);
        }

        // Feature: file-encryption, Property 6: Wrong password authentication failure
        // For any password-encrypted file, attempting to decrypt with an incorrect password
        // should fail authentication during the AES-GCM decryption process.

        #[test]
        fn prop_wrong_password_fails_authentication(
            data in prop::collection::vec(any::<u8>(), 1..1000),
            password1 in "[a-zA-Z0-9]{1,100}",
            password2 in "[a-zA-Z0-9]{1,100}",
        ) {
            prop_assume!(password1 != password2);

            let salt = PasswordEncryption::generate_salt().unwrap();
            let nonce = PasswordEncryption::generate_nonce().unwrap();
            let params = test_params();

            // Encrypt with password1
            let key1 = PasswordEncryption::derive_key(&password1, &salt, &params).unwrap();
            let encrypted = PasswordEncryption::encrypt(&data, &key1, &nonce).unwrap();

            // Try to decrypt with password2 (wrong password)
            let key2 = PasswordEncryption::derive_key(&password2, &salt, &params).unwrap();
            let result = PasswordEncryption::decrypt(&encrypted, &key2, &nonce);

            // Should fail with authentication error
            assert!(result.is_err());
            if let Err(e) = result {
                assert!(matches!(e, CryptoError::AuthenticationFailed));
            }
        }

        // Feature: file-encryption, Property 9: Random value uniqueness
        // For any sequence of encryption operations, generated random values (nonces, symmetric keys, salts)
        // should be unique across operations with overwhelming probability.

        #[test]
        fn prop_random_salts_are_unique(_seed in any::<u8>()) {
            // Generate multiple salts
            let salts: Vec<[u8; 32]> = (0..100)
                .map(|_| PasswordEncryption::generate_salt().unwrap())
                .collect();

            // Check that all salts are unique
            for i in 0..salts.len() {
                for j in (i + 1)..salts.len() {
                    assert_ne!(salts[i], salts[j], "Salts at positions {} and {} are identical", i, j);
                }
            }
        }

        #[test]
        fn prop_random_nonces_are_unique(_seed in any::<u8>()) {
            // Generate multiple nonces
            let nonces: Vec<[u8; 12]> = (0..100)
                .map(|_| PasswordEncryption::generate_nonce().unwrap())
                .collect();

            // Check that all nonces are unique
            for i in 0..nonces.len() {
                for j in (i + 1)..nonces.len() {
                    assert_ne!(nonces[i], nonces[j], "Nonces at positions {} and {} are identical", i, j);
                }
            }
        }
    }
}
