# Implementation Plan

- [x] 1. Set up cryptography dependencies and core types
  - Add required dependencies to Cargo.toml (ring, rsa, argon2, pem, zeroize, proptest)
  - Create crypto module structure (mod.rs, password.rs, rsa.rs, container.rs, keys.rs)
  - Define core encryption types (EncryptionMethod, DecryptionMethod, EncryptionType, EncryptionMetadata)
  - Define CryptoError enum with all error variants
  - _Requirements: 7.1, 7.3, 7.5_

- [ ] 2. Implement encrypted container format
  - [x] 2.1 Create EncryptedContainer struct and serialization logic
    - Implement container structure with magic bytes, version, encryption type, metadata, and encrypted data
    - Write serialization method to convert container to bytes
    - Write deserialization method to parse bytes into container
    - _Requirements: 1.4, 2.5_

  - [x] 2.2 Write property test for container round-trip
    - **Property 3: Encryption metadata completeness**
    - **Validates: Requirements 1.4, 2.5, 7.4**

  - [x] 2.3 Implement metadata structures for password and RSA encryption
    - Create PasswordMetadata with salt, nonce, and Argon2 parameters
    - Create RsaMetadata with encrypted key and nonce
    - Implement serialization/deserialization for both metadata types
    - _Requirements: 1.4, 2.5, 7.4_

  - [x] 2.4 Write unit tests for metadata serialization
    - Test password metadata serialization/deserialization
    - Test RSA metadata serialization/deserialization
    - Test metadata size calculations
    - _Requirements: 1.4, 2.5_

- [ ] 3. Implement password-based encryption
  - [x] 3.1 Implement Argon2id key derivation
    - Create key derivation function using Argon2id with secure parameters
    - Implement salt generation using cryptographically secure RNG
    - Add password validation (reject empty passwords)
    - _Requirements: 1.2, 1.5, 7.1, 7.2_

  - [x] 3.2 Implement AES-256-GCM encryption for password mode
    - Create encryption function that takes data and derived key
    - Generate random nonce for each encryption
    - Implement AES-256-GCM encryption with authentication
    - _Requirements: 1.3, 7.3_

  - [x] 3.3 Implement AES-256-GCM decryption for password mode
    - Create decryption function that takes encrypted data and derived key
    - Extract nonce from metadata
    - Implement AES-256-GCM decryption with authentication verification
    - _Requirements: 3.4, 3.5_

  - [x] 3.4 Write property test for password encryption round-trip
    - **Property 1: Password encryption round-trip**
    - **Validates: Requirements 1.2, 1.3, 1.4, 3.3, 3.4**

  - [x] 3.5 Write property test for wrong password authentication failure
    - **Property 6: Wrong password authentication failure**
    - **Validates: Requirements 3.5**

  - [x] 3.6 Write property test for random value uniqueness
    - **Property 9: Random value uniqueness**
    - **Validates: Requirements 7.2**

- [ ] 4. Implement RSA encryption
  - [x] 4.1 Implement RSA key file parsing
    - Create function to read and parse PEM-formatted private keys
    - Create function to read and parse PEM-formatted public keys
    - Add key file validation (existence, readability)
    - _Requirements: 2.1, 2.2, 4.1, 4.2_

  - [x] 4.2 Write property test for key file validation
    - **Property 4: Key file validation**
    - **Validates: Requirements 2.1, 4.1**

  - [x] 4.3 Write property test for PEM key parsing
    - **Property 5: PEM key parsing**
    - **Validates: Requirements 2.2, 4.2**

  - [x] 4.4 Implement RSA key size and padding validation
    - Validate RSA key size is at least 2048 bits
    - Configure RSA operations to use OAEP padding with SHA-256
    - _Requirements: 7.5_

  - [x] 4.5 Write property test for RSA key size and padding validation
    - **Property 10: RSA key size and padding validation**
    - **Validates: Requirements 7.5**

  - [x] 4.6 Implement symmetric key generation and RSA encryption
    - Generate random 256-bit symmetric key for AES-256-GCM
    - Encrypt symmetric key using RSA private key with OAEP padding
    - Encrypt file data using AES-256-GCM with generated symmetric key
    - _Requirements: 2.3, 2.4, 2.5_

  - [x] 4.7 Implement RSA decryption and symmetric key recovery
    - Extract encrypted symmetric key from metadata
    - Decrypt symmetric key using RSA public key
    - Decrypt file data using recovered symmetric key with AES-256-GCM
    - _Requirements: 4.3, 4.4, 4.5_

  - [x] 4.8 Write property test for RSA encryption round-trip
    - **Property 2: RSA encryption round-trip**
    - **Validates: Requirements 2.3, 2.4, 2.5, 4.3, 4.4, 4.5**

- [ ] 5. Extend CLI arguments for encryption options
  - [x] 5.1 Add encryption-related CLI arguments
    - Add --encrypt-password flag for password-based encryption
    - Add --encrypt-rsa option for RSA private key path
    - Add --decrypt-rsa option for RSA public key path
    - Update CliArgs struct and validation
    - _Requirements: 8.1, 8.2, 8.3_

  - [x] 5.2 Implement CLI argument validation
    - Validate that password and RSA options are mutually exclusive
    - Validate key file paths when provided
    - Add appropriate error messages for invalid combinations
    - _Requirements: 8.4_

  - [x] 5.3 Write unit tests for CLI argument validation
    - Test mutual exclusivity of password and RSA options
    - Test key file path validation
    - Test error messages for invalid combinations
    - _Requirements: 8.4_

- [ ] 6. Extend configuration types for encryption
  - [x] 6.1 Extend CompressionConfig with encryption options
    - Add encryption field to CompressionConfig
    - Create EncryptionMethod enum (Password, Rsa)
    - Update config builder methods
    - _Requirements: 5.1, 5.3_

  - [x] 6.2 Create DecompressionConfig with decryption options
    - Create DecompressionConfig struct
    - Add decryption field with DecryptionMethod enum
    - Implement config builder methods
    - _Requirements: 3.1, 4.1_

  - [x] 6.3 Write property test for compression options preservation
    - **Property 7: Compression options preservation**
    - **Validates: Requirements 5.3**

- [ ] 7. Implement encryption operations
  - [x] 7.1 Create encrypt_file function
    - Implement function that takes compressed file and encryption config
    - Route to password or RSA encryption based on config
    - Generate output filename with .jcze extension
    - Write encrypted container to output file
    - _Requirements: 5.1, 5.4_

  - [x] 7.2 Write property test for file extension indication
    - **Property 13: File extension indication**
    - **Validates: Requirements 5.4**

  - [x] 7.3 Create encrypt_files function for batch processing
    - Implement parallel encryption of multiple files using rayon
    - Ensure each file gets independent encryption keys
    - Handle errors for individual files without stopping batch
    - _Requirements: 5.5_

  - [x] 7.4 Write property test for independent file encryption
    - **Property 8: Independent file encryption**
    - **Validates: Requirements 5.5**

  - [x] 7.5 Add encryption logging and user feedback
    - Log encryption method at start of operation
    - Display confirmation message with output path on success
    - Display specific error messages on failure
    - _Requirements: 6.1, 6.2, 6.5_

  - [x] 7.6 Write property test for error message specificity
    - **Property 14: Error message specificity**
    - **Validates: Requirements 6.5**

- [ ] 8. Implement decryption operations
  - [x] 8.1 Create decrypt_file function
    - Implement function to read and parse encrypted container
    - Detect encryption type from metadata
    - Route to password or RSA decryption based on type
    - Write decrypted data to output file
    - _Requirements: 3.1, 5.2_

  - [x] 8.2 Write property test for encrypted file detection
    - **Property 12: Encrypted file detection**
    - **Validates: Requirements 3.1**

  - [x] 8.3 Implement password prompting for decryption
    - Create secure password prompt function (no echo)
    - Prompt user when password-encrypted file is detected
    - Handle password input errors gracefully
    - _Requirements: 3.2_

  - [x] 8.4 Create decrypt_files function for batch processing
    - Implement parallel decryption of multiple files
    - Handle mixed encrypted/non-encrypted files
    - Prompt for passwords as needed
    - _Requirements: 5.2, 8.5_

  - [x] 8.5 Write property test for non-encrypted file handling
    - **Property 11: Non-encrypted file handling**
    - **Validates: Requirements 8.5**

  - [x] 8.6 Add decryption logging and user feedback
    - Log detected encryption method at start
    - Display confirmation message on successful decryption
    - Display specific error messages on failure
    - _Requirements: 6.3, 6.4, 6.5_

- [ ] 9. Integrate encryption with compression workflow
  - [x] 9.1 Modify compress operations to support encryption
    - Update compress_file to optionally encrypt after compression
    - Update compress_files to handle encryption in parallel
    - Ensure compression options (level, timestamp, move-to) work with encryption
    - _Requirements: 5.1, 5.3_

  - [x] 9.2 Modify decompress operations to support decryption
    - Update decompress_file to detect and decrypt encrypted files
    - Update decompress_files to handle decryption automatically
    - Ensure decompression works seamlessly after decryption
    - _Requirements: 5.2_

  - [x] 9.3 Update CLI command execution
    - Wire encryption options from CLI args to compression config
    - Wire decryption options from CLI args to decompression config
    - Update command handlers to use new encryption/decryption functions
    - _Requirements: 8.1, 8.2, 8.3_

- [x] 10. Write integration tests
  - Test end-to-end password encryption/decryption with all compression formats
  - Test end-to-end RSA encryption/decryption with all compression formats
  - Test encryption with timestamps and move-to directory options
  - Test batch encryption/decryption of multiple files
  - Test error handling for various failure scenarios
  - _Requirements: All_

- [ ] 11. Implement secure memory handling
  - [x] 11.1 Add zeroization for sensitive data
    - Zeroize passwords after key derivation
    - Zeroize symmetric keys after use
    - Zeroize derived keys after encryption/decryption
    - _Requirements: 7.1, 7.2_

  - [x] 11.2 Write unit tests for memory zeroization
    - Verify passwords are zeroized after use
    - Verify keys are zeroized after use
    - Test zeroization on error paths
    - _Requirements: 7.1, 7.2_

- [x] 12. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
