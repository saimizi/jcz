use rayon::prelude::*;
use std::path::PathBuf;

use crate::compressors::create_compressor;
use crate::core::config::CompressionConfig;
use crate::core::config::TimestampOption;
use crate::core::error::JcResult;
use crate::core::types::CompoundFormat;
use crate::operations::encrypt;
use crate::utils::{debug, info, remove_file_silent};

/// Compress file(s) with compound format (TAR + secondary compression)
pub fn compress_compound(
    input: &PathBuf,
    format: CompoundFormat,
    config: &CompressionConfig,
) -> JcResult<PathBuf> {
    info!(
        "Compressing {} with compound format: {}",
        input.display(),
        format.extension()
    );

    // Step 1: Create TAR archive
    let tar_compressor = create_compressor(format.primary());
    let tar_config = CompressionConfig {
        level: 0, // TAR doesn't use compression level
        timestamp: config.timestamp,
        move_to: None, // Don't move intermediate file
        show_output_size: false,
        force: config.force,
        encryption: None, // Encryption happens after compound compression
    };

    // Remove timestamp to avoid duplication
    let new_config = config.clone().with_timestamp(TimestampOption::None);

    let tar_output = tar_compressor.compress(input, &tar_config)?;
    debug!("Created intermediate TAR: {}", tar_output.display());

    // Step 2: Compress TAR with secondary compressor
    let secondary_compressor = create_compressor(format.secondary());
    let secondary_output = secondary_compressor.compress(&tar_output, &new_config)?;

    // Step 3: Remove intermediate TAR file
    if let Err(e) = remove_file_silent(&tar_output) {
        debug!("Failed to remove intermediate TAR: {}", e);
    }

    info!("Created compound archive: {}", secondary_output.display());

    // Step 4: Encrypt if encryption is enabled
    if let Some(encryption_method) = &config.encryption {
        encrypt::encrypt_file(&secondary_output, encryption_method)
    } else {
        Ok(secondary_output)
    }
}

/// Compress multiple files with compound format
pub fn compress_compound_batch(
    inputs: Vec<PathBuf>,
    format: CompoundFormat,
    config: CompressionConfig,
) -> Vec<JcResult<PathBuf>> {
    // Check if password encryption is used
    let has_password_encryption = matches!(
        config.encryption,
        Some(crate::core::config::EncryptionMethod::Password)
    );

    if has_password_encryption {
        // For password encryption, compress all files first, then encrypt with shared password
        let compressed: Vec<JcResult<PathBuf>> = inputs
            .par_iter()
            .map(|input| {
                // Compress without encryption first
                let mut temp_config = config.clone();
                temp_config.encryption = None;
                compress_compound(input, format, &temp_config)
            })
            .collect();

        // Collect successful compressions
        let compressed_paths: Vec<PathBuf> =
            compressed.into_iter().filter_map(|r| r.ok()).collect();

        // Encrypt all with the same password
        if let Some(encryption_method) = &config.encryption {
            encrypt::encrypt_files(compressed_paths, encryption_method)
        } else {
            vec![]
        }
    } else {
        // For RSA or no encryption, process independently
        inputs
            .par_iter()
            .map(|input| compress_compound(input, format, &config))
            .collect()
    }
}
