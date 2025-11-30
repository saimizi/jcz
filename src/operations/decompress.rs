use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;

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
        // Otherwise, create a subdirectory based on the archive name
        if current_file.is_dir() && current_file == temp_dir_path {
            // Multiple files extracted - put them directly in move_to
            move_to.clone()
        } else {
            // Single file or directory - create subdirectory
            let mut dest = input.clone();
            while detect_format(&dest).is_some() {
                dest = dest.with_extension("");
            }
            let filename = dest
                .file_name()
                .ok_or_else(|| JcError::Other("Invalid output filename".to_string()))?;
            move_to.join(filename)
        }
    } else {
        // Determine output based on input filename
        let mut dest = input.clone();
        // Remove all compression extensions
        while detect_format(&dest).is_some() {
            dest = dest.with_extension("");
        }
        dest
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
