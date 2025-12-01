use std::fs;
use std::path::PathBuf;

use crate::compressors::{create_compressor, tar::TarCompressor};
use crate::core::compressor::{Compressor, MultiFileCompressor};
use crate::core::config::{CollectionConfig, CollectionMode, CompressionConfig, TimestampOption};
use crate::core::error::{JcError, JcResult};
use crate::core::types::CompoundFormat;
use crate::operations::encrypt::encrypt_file;
use crate::utils::{copy_recursive, create_temp_dir, debug, info, move_file, remove_file_silent};

/// Collect multiple files into a compressed archive
pub fn collect_and_compress(
    inputs: Vec<PathBuf>,
    format: CompoundFormat,
    collection_config: CollectionConfig,
) -> JcResult<PathBuf> {
    // Validate inputs
    if inputs.is_empty() {
        return Err(JcError::NoInputFiles);
    }

    // Check for duplicate basenames
    let basenames: Vec<String> = inputs
        .iter()
        .filter_map(|p| p.file_name())
        .filter_map(|n| n.to_str())
        .map(|s| s.to_string())
        .collect();

    let mut unique_basenames = basenames.clone();
    unique_basenames.sort();
    unique_basenames.dedup();

    if basenames.len() != unique_basenames.len() {
        let duplicates: Vec<String> = basenames
            .iter()
            .filter(|name| basenames.iter().filter(|n| *n == *name).count() > 1)
            .cloned()
            .collect();
        return Err(JcError::DuplicateBasenames(duplicates));
    }

    // Check package name doesn't exist
    let package_path = PathBuf::from(&collection_config.package_name);
    if package_path.exists() {
        return Err(JcError::NameExists(collection_config.package_name.clone()));
    }

    info!(
        "Collecting {} files into {}",
        inputs.len(),
        collection_config.package_name
    );

    // Create temporary staging directory
    let temp_dir = create_temp_dir("jczpkg_")?;
    debug!("Created temporary directory: {}", temp_dir.display());

    // Ensure cleanup on exit
    let _cleanup = CleanupGuard::new(temp_dir.clone());

    let staging_dir = match collection_config.mode {
        CollectionMode::WithParent => {
            // Create subdirectory with package name
            let pkg_dir = temp_dir.join(&collection_config.package_name);
            fs::create_dir(&pkg_dir)?;
            pkg_dir
        }
        CollectionMode::Flat => {
            // Use temp dir directly
            temp_dir.clone()
        }
    };

    // Copy files to staging directory
    for input in &inputs {
        let basename = input
            .file_name()
            .ok_or_else(|| JcError::Other("Invalid filename".to_string()))?;
        let dest = staging_dir.join(basename);

        debug!("Copying {} to {}", input.display(), dest.display());
        copy_recursive(input, &dest)?;
    }

    // Create TAR archive
    let tar_compressor = TarCompressor::new();

    let tar_config = CompressionConfig {
        level: 0,
        timestamp: collection_config.base.timestamp,
        move_to: None,
        show_output_size: false,
        force: collection_config.base.force,
        encryption: None, // Encryption happens after collection
    };

    // Generate TAR filename
    let tar_filename = if collection_config.mode == CollectionMode::Flat {
        // For flat mode, create TAR from staging dir contents
        let file_list: Vec<PathBuf> = inputs
            .iter()
            .map(|p| staging_dir.join(p.file_name().unwrap()))
            .collect();

        tar_compressor.compress_multi(&file_list, &collection_config.package_name, &tar_config)?
    } else {
        // Archive the package directory
        let archive_input = temp_dir.join(&collection_config.package_name);
        tar_compressor.compress(&archive_input, &tar_config)?
    };

    debug!("Created TAR archive: {}", tar_filename.display());

    // Apply secondary compression
    let final_output = if format.secondary() != format.primary() {
        let secondary_compressor = create_compressor(format.secondary());

        // Remove timestamp to avoid duplication
        let new_config = collection_config
            .base
            .clone()
            .with_timestamp(TimestampOption::None);
        let compressed = secondary_compressor.compress(&tar_filename, &new_config)?;

        // Remove intermediate TAR
        let _ = remove_file_silent(&tar_filename);

        compressed
    } else {
        tar_filename
    };

    // Apply encryption if specified
    let final_output = if let Some(ref encryption_method) = collection_config.base.encryption {
        encrypt_file(&final_output, encryption_method)?
    } else {
        final_output
    };

    // Move to destination or current directory
    let destination = collection_config
        .base
        .move_to
        .unwrap_or_else(|| PathBuf::from("."));

    let final_path = move_file(&final_output, &destination)?;

    info!("Created collection archive: {}", final_path.display());
    Ok(final_path)
}

/// RAII guard for cleaning up temporary directory
struct CleanupGuard {
    path: PathBuf,
}

impl CleanupGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_dir_all(&self.path) {
            debug!(
                "Failed to cleanup temp directory {}: {}",
                self.path.display(),
                e
            );
        } else {
            debug!("Cleaned up temp directory: {}", self.path.display());
        }
    }
}
