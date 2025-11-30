use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::compressor::Compressor;
use crate::core::config::CompressionConfig;
use crate::core::error::{JcError, JcResult};
use crate::utils::{copy_to_dir, debug, generate_output_filename, info, move_file_if_needed};

/// ZIP compressor/decompressor implementation
#[derive(Debug, Clone)]
pub struct ZipCompressor;

impl ZipCompressor {
    pub fn new() -> Self {
        Self
    }

    /// Validate that input exists
    fn validate_input(&self, path: &Path) -> JcResult<()> {
        if !path.exists() {
            return Err(JcError::FileNotFound(path.to_path_buf()));
        }
        Ok(())
    }
}

impl Compressor for ZipCompressor {
    fn name(&self) -> &'static str {
        "zip"
    }

    fn extension(&self) -> &'static str {
        "zip"
    }

    fn compress(&self, input: &Path, config: &CompressionConfig) -> JcResult<PathBuf> {
        self.validate_input(input)?;

        let output_path = generate_output_filename(input, "zip", config.timestamp)?;
        info!(
            "Compressing {} to {} with zip",
            input.display(),
            output_path.display()
        );
        debug!("Compression level: {}", config.level);

        // Build zip command
        let mut cmd = Command::new("zip");

        // Add compression level (0-9)
        cmd.arg(format!("-{}", config.level));

        // Recursive flag for directories
        if input.is_dir() {
            cmd.arg("-r");
        }

        // Quiet mode
        cmd.arg("-q");

        // Output file and input
        cmd.arg(&output_path).arg(input);

        debug!("Executing: {:?}", cmd);

        let output = cmd
            .output()
            .map_err(|e| JcError::Other(format!("Failed to execute zip: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JcError::CompressionFailed {
                tool: "zip".to_string(),
                stderr: stderr.to_string(),
            });
        }

        // Move to destination if specified
        let final_path = move_file_if_needed(&output_path, &config.move_to)?;

        info!("Compressed file: {}", final_path.display());
        Ok(final_path)
    }

    fn decompress(&self, input: &Path, _config: &CompressionConfig) -> JcResult<PathBuf> {
        // Validate extension
        if !input.to_string_lossy().ends_with(".zip") {
            return Err(JcError::InvalidExtension(
                input.to_path_buf(),
                "zip".to_string(),
            ));
        }

        debug!("Decompressing {} with unzip", input.display());

        let parent = input.parent().unwrap_or_else(|| Path::new("."));

        // Execute unzip command
        let mut cmd = Command::new("unzip");
        cmd.arg("-o") // overwrite without prompting
            .arg(input)
            .arg("-d")
            .arg(parent);

        let output = cmd
            .output()
            .map_err(|e| JcError::Other(format!("Failed to execute unzip: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JcError::DecompressionFailed {
                tool: "unzip".to_string(),
                stderr: stderr.to_string(),
            });
        }

        // Output is the filename without .zip extension
        let output_path = input.with_extension("");

        info!("Decompressed ZIP archive: {}", output_path.display());
        Ok(output_path)
    }

    fn supports_levels(&self) -> bool {
        true
    }

    fn validate_level(&self, level: u8) -> bool {
        level <= 9 // ZIP supports 0-9
    }

    fn default_level(&self) -> u8 {
        6
    }
}

impl ZipCompressor {
    /// Decompress in a specific working directory
    pub fn decompress_in_dir(
        &self,
        input: &Path,
        working_dir: &Path,
        _config: &CompressionConfig,
    ) -> JcResult<PathBuf> {
        // Validate extension
        if !input.to_string_lossy().ends_with(".zip") {
            return Err(JcError::InvalidExtension(
                input.to_path_buf(),
                "zip".to_string(),
            ));
        }

        debug!(
            "Decompressing {} with unzip in working dir {}",
            input.display(),
            working_dir.display()
        );

        // Copy input file to working directory
        let work_input = copy_to_dir(input, working_dir)?;

        // Execute unzip command in working directory
        let mut cmd = Command::new("unzip");
        cmd.arg("-o") // overwrite without prompting
            .arg(&work_input)
            .arg("-d")
            .arg(working_dir);

        let output = cmd
            .output()
            .map_err(|e| JcError::Other(format!("Failed to execute unzip: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JcError::DecompressionFailed {
                tool: "unzip".to_string(),
                stderr: stderr.to_string(),
            });
        }

        // Find what was extracted (similar to TAR behavior)
        use std::fs;
        let entries: Vec<_> = fs::read_dir(working_dir)
            .map_err(|e| JcError::Io(e))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path() != work_input) // Exclude the zip file itself
            .collect();

        // Remove the copied zip file from working directory
        let _ = fs::remove_file(&work_input);

        // If we found exactly one entry, use that
        if entries.len() == 1 {
            let extracted_path = entries[0].path();
            debug!("Extracted to: {}", extracted_path.display());
            return Ok(extracted_path);
        }

        // Check if there's a directory with the zip's base name
        let zip_base_name = work_input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        for entry in &entries {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) {
                    if dir_name == zip_base_name {
                        debug!("Extracted to directory: {}", path.display());
                        return Ok(path);
                    }
                }
            }
        }

        // Multiple files extracted - return the working directory
        if !entries.is_empty() {
            debug!(
                "Extracted {} files to: {}",
                entries.len(),
                working_dir.display()
            );
            return Ok(working_dir.to_path_buf());
        }

        // Fallback: assume filename without .zip extension
        let output_path = work_input.with_extension("");
        debug!("Extracted to (fallback): {}", output_path.display());
        Ok(output_path)
    }
}
