use std::path::PathBuf;

use crate::cli::args::CliArgs;
use crate::core::config::{
    CollectionConfig, CollectionMode, CompressionConfig, DecryptionMethod, EncryptionMethod,
    TimestampOption,
};
use crate::core::error::{JcError, JcResult};
use crate::core::types::{CompoundFormat, CompressionFormat};
use crate::operations::{collect_and_compress, compound, compress, decompress};
use crate::utils::{error, validate_input_files, validate_move_to};

/// Execute the appropriate command based on CLI arguments
pub fn execute(args: CliArgs) -> JcResult<()> {
    // Validate arguments
    args.validate().map_err(JcError::Other)?;

    // Build configuration
    let timestamp = TimestampOption::from_u8(args.timestamp)
        .ok_or(JcError::InvalidTimestampOption(args.timestamp))?;

    let config = CompressionConfig::new()
        .with_level(args.level)
        .with_timestamp(timestamp)
        .with_force(args.force);

    let config = if let Some(ref move_to) = args.move_to {
        validate_move_to(move_to)?;
        config.with_move_to(move_to.clone())
    } else {
        config
    };

    // Add encryption configuration if specified
    let config = if args.encrypt_password {
        config.with_encryption(Some(EncryptionMethod::Password))
    } else if let Some(ref public_key_path) = args.encrypt_key {
        config.with_encryption(Some(EncryptionMethod::Rsa {
            public_key_path: public_key_path.clone(),
        }))
    } else {
        config
    };

    // Validate input files
    let inputs = validate_input_files(args.inputs)?;
    let input_paths: Vec<PathBuf> = inputs.iter().map(|f| f.real_path.clone()).collect();

    if args.decompress {
        // Decompression mode
        let decryption_method = if let Some(ref private_key_path) = args.decrypt_key {
            Some(DecryptionMethod::Rsa {
                private_key_path: private_key_path.clone(),
            })
        } else {
            None
        };
        handle_decompress(
            input_paths,
            config,
            decryption_method,
            args.remove_encrypted,
        )
    } else if args.collect.is_some() || args.collect_flat.is_some() {
        // Collection mode
        let mode = if args.collect.is_some() {
            CollectionMode::WithParent
        } else {
            CollectionMode::Flat
        };

        let package_name = args.collect.or(args.collect_flat).unwrap();

        handle_collection(input_paths, &args.command, package_name, mode, config)
    } else {
        // Standard compression mode
        handle_compress(input_paths, &args.command, config)
    }
}

fn handle_decompress(
    inputs: Vec<PathBuf>,
    config: CompressionConfig,
    decryption_method: Option<DecryptionMethod>,
    remove_encrypted: bool,
) -> JcResult<()> {
    let results = decompress::decompress_files(inputs, config, decryption_method, remove_encrypted);

    // Check for errors
    let mut had_errors = false;
    for result in results {
        if let Err(e) = result {
            error!("Decompression failed: {}", e);
            had_errors = true;
        }
    }

    if had_errors {
        Err(JcError::Other(
            "Some files failed to decompress".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn handle_compress(inputs: Vec<PathBuf>, command: &str, config: CompressionConfig) -> JcResult<()> {
    // Determine if simple or compound format
    if let Some(compound) = CompoundFormat::from_str(command) {
        // Compound format (tgz, tbz2, txz)
        let results = compound::compress_compound_batch(inputs, compound, config);

        let mut had_errors = false;
        for result in results {
            if let Err(e) = result {
                error!("Compression failed: {}", e);
                had_errors = true;
            }
        }

        if had_errors {
            Err(JcError::Other("Some files failed to compress".to_string()))
        } else {
            Ok(())
        }
    } else {
        // Simple format (gzip, bzip2, xz, tar)
        let format = CompressionFormat::from_name(command)
            .ok_or_else(|| JcError::InvalidCommand(command.to_string()))?;

        let results = compress::compress_files(inputs, format, config);

        let mut had_errors = false;
        for result in results {
            if let Err(e) = result {
                error!("Compression failed: {}", e);
                had_errors = true;
            }
        }

        if had_errors {
            Err(JcError::Other("Some files failed to compress".to_string()))
        } else {
            Ok(())
        }
    }
}

fn handle_collection(
    inputs: Vec<PathBuf>,
    command: &str,
    package_name: String,
    mode: CollectionMode,
    config: CompressionConfig,
) -> JcResult<()> {
    let compound = CompoundFormat::from_str(command)
        .ok_or_else(|| JcError::InvalidCommand(command.to_string()))?;

    let collection_config = CollectionConfig {
        base: config,
        package_name,
        mode,
    };

    collect_and_compress(inputs, compound, collection_config)?;

    Ok(())
}
