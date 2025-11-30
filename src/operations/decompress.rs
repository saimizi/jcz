use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::compressors::{
    detect_format, Bzip2Compressor, GzipCompressor, TarCompressor, XzCompressor, ZipCompressor,
};
use crate::core::config::CompressionConfig;
use crate::core::error::{JcError, JcResult};
use crate::utils::{create_decompress_temp_dir, debug, error, info, prompt_overwrite};

/// Helper function to decompress in a working directory based on format
fn decompress_in_working_dir(
    format: crate::core::types::CompressionFormat,
    input: &PathBuf,
    working_dir: &PathBuf,
    config: &CompressionConfig,
) -> JcResult<PathBuf> {
    use crate::core::types::CompressionFormat;

    match format {
        CompressionFormat::Gzip => {
            let compressor = GzipCompressor::new();
            compressor.decompress_in_dir(input, working_dir, config)
        }
        CompressionFormat::Bzip2 => {
            let compressor = Bzip2Compressor::new();
            compressor.decompress_in_dir(input, working_dir, config)
        }
        CompressionFormat::Xz => {
            let compressor = XzCompressor::new();
            compressor.decompress_in_dir(input, working_dir, config)
        }
        CompressionFormat::Tar => {
            let compressor = TarCompressor::new();
            compressor.decompress_in_dir(input, working_dir, config)
        }
        CompressionFormat::Zip => {
            let compressor = ZipCompressor::new();
            compressor.decompress_in_dir(input, working_dir, config)
        }
    }
}

/// Decompress a single file, handling compound formats
pub fn decompress_file(input: &PathBuf, config: &CompressionConfig) -> JcResult<PathBuf> {
    // Create a temporary directory for decompression work
    let temp_dir = create_decompress_temp_dir()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    debug!("Created temp directory: {}", temp_dir_path.display());

    let mut current_file = input.clone();

    // Iteratively decompress until no more compression detected
    loop {
        let format = detect_format(&current_file).ok_or_else(|| {
            JcError::InvalidExtension(
                current_file.clone(),
                "supported compression format".to_string(),
            )
        })?;

        info!(
            "Decompression iteration: format={:?}, current_file={}",
            format,
            current_file.display()
        );

        // Decompress in temp directory
        let output = decompress_in_working_dir(format, &current_file, &temp_dir_path, config)?;

        info!("Decompressed to: {}", output.display());

        current_file = output;

        // Check if output has another compression layer
        if detect_format(&current_file).is_none() {
            info!("No more compression layers detected");
            break;
        }
    }

    // Determine final destination
    let final_dest = if let Some(ref move_to) = config.move_to {
        // When using -C with multiple extracted files, use move_to directly
        // Otherwise, create a subdirectory based on the actual extracted content name
        if current_file.is_dir() && current_file == temp_dir_path {
            // Multiple files extracted - put them directly in move_to
            move_to.clone()
        } else {
            // Single file or directory - use the actual extracted content name
            let extracted_name = current_file
                .file_name()
                .ok_or_else(|| JcError::Other("Invalid extracted filename".to_string()))?;
            move_to.join(extracted_name)
        }
    } else {
        // Use the actual extracted content name, not the archive name
        // current_file points to the extracted content in temp directory
        let extracted_name = current_file
            .file_name()
            .ok_or_else(|| JcError::Other("Invalid extracted filename".to_string()))?;

        // Place it in the same directory as the input archive
        let input_parent = input.parent().unwrap_or_else(|| Path::new("."));
        input_parent.join(extracted_name)
    };

    debug!("Final destination: {}", final_dest.display());

    // Move from temp directory to final destination
    // All decompressed files are in temp directory, so we always need to copy/move them
    if current_file.is_dir() {
        // This is a directory containing multiple extracted files
        // We need to copy the contents, not create a subdirectory
        if current_file == temp_dir_path {
            // This is the working directory itself (multiple loose files from TAR)
            // Copy contents to final destination
            fs::create_dir_all(&final_dest).map_err(|e| JcError::Io(e))?;
            for entry in fs::read_dir(&current_file).map_err(|e| JcError::Io(e))? {
                let entry = entry.map_err(|e| JcError::Io(e))?;
                let src_path = entry.path();
                let dst_path = final_dest.join(entry.file_name());

                // Check if individual file exists and prompt for overwrite
                if dst_path.exists() && !config.force {
                    if !prompt_overwrite(&dst_path)? {
                        info!("Skipping {}", dst_path.display());
                        continue;
                    }
                }

                use crate::utils::copy_recursive;
                if src_path.is_dir() {
                    copy_recursive(&src_path, &dst_path).map_err(|e| JcError::Io(e))?;
                } else {
                    fs::copy(&src_path, &dst_path).map_err(|e| JcError::Io(e))?;
                }
            }
            info!(
                "Decompressed {} files to: {}",
                fs::read_dir(&final_dest).map(|d| d.count()).unwrap_or(0),
                final_dest.display()
            );
        } else {
            // This is a subdirectory that was extracted from TAR
            // Check if destination exists
            if final_dest.exists() && !config.force {
                if !prompt_overwrite(&final_dest)? {
                    return Err(JcError::Other(format!(
                        "Decompression aborted: directory already exists: {}",
                        final_dest.display()
                    )));
                }
                // Remove existing directory/file before copying
                if final_dest.is_dir() {
                    fs::remove_dir_all(&final_dest).map_err(|e| JcError::Io(e))?;
                } else {
                    fs::remove_file(&final_dest).map_err(|e| JcError::Io(e))?;
                }
            } else if final_dest.exists() && config.force {
                // Force mode: remove without prompting
                if final_dest.is_dir() {
                    fs::remove_dir_all(&final_dest).map_err(|e| JcError::Io(e))?;
                } else {
                    fs::remove_file(&final_dest).map_err(|e| JcError::Io(e))?;
                }
            }
            use crate::utils::copy_recursive;
            copy_recursive(&current_file, &final_dest).map_err(|e| JcError::Io(e))?;
            info!("Decompressed directory: {}", final_dest.display());
        }
    } else {
        // Copy single file
        // Check if destination exists
        if final_dest.exists() && !config.force {
            if !prompt_overwrite(&final_dest)? {
                return Err(JcError::Other(format!(
                    "Decompression aborted: file already exists: {}",
                    final_dest.display()
                )));
            }
            // Remove existing file/directory before copying
            if final_dest.is_dir() {
                fs::remove_dir_all(&final_dest).map_err(|e| JcError::Io(e))?;
            } else {
                fs::remove_file(&final_dest).map_err(|e| JcError::Io(e))?;
            }
        } else if final_dest.exists() && config.force {
            // Force mode: remove without prompting
            if final_dest.is_dir() {
                fs::remove_dir_all(&final_dest).map_err(|e| JcError::Io(e))?;
            } else {
                fs::remove_file(&final_dest).map_err(|e| JcError::Io(e))?;
            }
        }
        fs::copy(&current_file, &final_dest).map_err(|e| JcError::Io(e))?;
        info!("Decompressed file: {}", final_dest.display());
    }

    // temp_dir will be automatically cleaned up when it goes out of scope
    Ok(final_dest)
}

/// Decompress multiple files concurrently
pub fn decompress_files(inputs: Vec<PathBuf>, config: CompressionConfig) -> Vec<JcResult<PathBuf>> {
    info!("Decompressing {} files", inputs.len());

    inputs
        .par_iter()
        .map(|input| match decompress_file(input, &config) {
            Ok(output) => Ok(output),
            Err(e) => {
                error!("Failed to decompress {}: {}", input.display(), e);
                Err(e)
            }
        })
        .collect()
}
