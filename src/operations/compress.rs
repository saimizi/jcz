use rayon::prelude::*;
use std::path::PathBuf;

use crate::compressors::create_compressor;
use crate::core::config::CompressionConfig;
use crate::core::error::{JcError, JcResult};
use crate::core::types::CompressionFormat;
use crate::operations::encrypt;
use crate::utils::{error, info};

/// Compress a single file
#[allow(dead_code)]
pub fn compress_file(
    input: &PathBuf,
    format: CompressionFormat,
    config: &CompressionConfig,
) -> JcResult<PathBuf> {
    let compressor = create_compressor(format);

    // Validate compression level if supported
    if compressor.supports_levels() && !compressor.validate_level(config.level) {
        return Err(JcError::InvalidCompressionLevel {
            algorithm: compressor.name().to_string(),
            level: config.level,
        });
    }

    let compressed_path = compressor.compress(input, config)?;

    // Encrypt if encryption is enabled
    if let Some(encryption_method) = &config.encryption {
        encrypt::encrypt_file(&compressed_path, encryption_method)
    } else {
        Ok(compressed_path)
    }
}

/// Compress multiple files concurrently
pub fn compress_files(
    inputs: Vec<PathBuf>,
    format: CompressionFormat,
    config: CompressionConfig,
) -> Vec<JcResult<PathBuf>> {
    info!("Compressing {} files with {}", inputs.len(), format.name());

    // Compress files first
    let compressed: Vec<JcResult<PathBuf>> = inputs
        .par_iter()
        .map(|input| {
            let compressor = create_compressor(format);
            if compressor.supports_levels() && !compressor.validate_level(config.level) {
                return Err(JcError::InvalidCompressionLevel {
                    algorithm: compressor.name().to_string(),
                    level: config.level,
                });
            }
            compressor.compress(input, &config).map_err(|e| {
                error!("Failed to compress {}: {}", input.display(), e);
                e
            })
        })
        .collect();

    // If encryption is enabled, encrypt all compressed files
    if let Some(encryption_method) = &config.encryption {
        let compressed_paths: Vec<PathBuf> =
            compressed.into_iter().filter_map(|r| r.ok()).collect();

        encrypt::encrypt_files(compressed_paths, encryption_method)
    } else {
        compressed
    }
}
