use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::core::config::TimestampOption;
use crate::core::error::{JcError, JcResult};
use crate::utils::timestamp::generate_timestamp;

/// Generate output filename with optional timestamp
pub fn generate_output_filename(
    input: &Path,
    extension: &str,
    timestamp_opt: TimestampOption,
) -> JcResult<PathBuf> {
    let mut filename = input.as_os_str().to_string_lossy().to_string();

    // Remove trailing slash if present
    if filename.ends_with('/') {
        filename.pop();
    }

    // Add timestamp if requested
    if timestamp_opt != TimestampOption::None {
        let ts = generate_timestamp(timestamp_opt);
        filename.push('_');
        filename.push_str(&ts);
    }

    // Add extension
    filename.push('.');
    filename.push_str(extension);

    Ok(PathBuf::from(filename))
}

/// Move file to destination directory if specified
pub fn move_file_if_needed(source: &Path, move_to: &Option<PathBuf>) -> JcResult<PathBuf> {
    if let Some(dest_dir) = move_to {
        move_file(source, dest_dir)
    } else {
        Ok(source.to_path_buf())
    }
}

/// Move file to destination directory
pub fn move_file(source: &Path, dest_dir: &Path) -> JcResult<PathBuf> {
    // Validate destination is a directory
    if !dest_dir.is_dir() {
        return Err(JcError::NotADirectory(dest_dir.to_path_buf()));
    }

    let filename = source
        .file_name()
        .ok_or_else(|| JcError::Other("Invalid source filename".to_string()))?;

    let dest_path = dest_dir.join(filename);

    // Try rename first (fast, atomic), fall back to copy+delete for cross-device
    match fs::rename(source, &dest_path) {
        Ok(_) => Ok(dest_path),
        Err(e) if e.raw_os_error() == Some(18) => {
            // EXDEV (cross-device link) - fall back to copy + delete
            fs::copy(source, &dest_path)?;
            fs::remove_file(source)?;
            Ok(dest_path)
        }
        Err(e) => Err(JcError::Io(e)),
    }
}

/// Recursively copy file or directory, preserving symlinks.
///
/// Symlinks are recreated as symlinks rather than followed, so dangling
/// symlinks (whose targets do not exist) are copied faithfully instead of
/// causing an I/O error.
pub fn copy_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    // Inspect the link itself, not its target, so symlinks are detected even
    // when they are dangling.
    let metadata = fs::symlink_metadata(src)?;

    if metadata.file_type().is_symlink() {
        copy_symlink(src, dst)?;
    } else if metadata.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            copy_recursive(&src_path, &dst_path)?;
        }
    } else {
        fs::copy(src, dst)?;
    }
    Ok(())
}

/// Recreate a symlink at `dst` pointing to the same target as `src`.
fn copy_symlink(src: &Path, dst: &Path) -> io::Result<()> {
    let target = fs::read_link(src)?;
    // Remove any existing entry at the destination so the symlink can be created.
    if fs::symlink_metadata(dst).is_ok() {
        if dst.is_dir() {
            fs::remove_dir_all(dst)?;
        } else {
            fs::remove_file(dst)?;
        }
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, dst)?;
    }
    #[cfg(windows)]
    {
        // Best-effort on Windows: choose the symlink kind based on the target.
        if target.is_dir() {
            std::os::windows::fs::symlink_dir(&target, dst)?;
        } else {
            std::os::windows::fs::symlink_file(&target, dst)?;
        }
    }
    Ok(())
}

/// Copy directory contents excluding specific files
#[allow(dead_code)]
pub fn copy_directory_contents_except(src: &Path, dst: &Path, exclude: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();

        // Skip the excluded file
        if src_path == exclude {
            continue;
        }

        let dst_path = dst.join(entry.file_name());
        copy_recursive(&src_path, &dst_path)?;
    }
    Ok(())
}

/// Remove file, ignoring errors
pub fn remove_file_silent(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// Create temporary directory with prefix
pub fn create_temp_dir(prefix: &str) -> JcResult<PathBuf> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let dir_name = format!("{}{:x}", prefix, timestamp);
    let temp_path = PathBuf::from(dir_name);

    fs::create_dir(&temp_path).map_err(|e| JcError::TempDirFailed(e.to_string()))?;

    Ok(temp_path)
}

/// Create a temporary directory for decompression work in /tmp
/// Returns a TempDir that will be automatically cleaned up when dropped
pub fn create_decompress_temp_dir() -> JcResult<TempDir> {
    TempDir::new_in("/tmp")
        .map_err(|e| JcError::TempDirFailed(format!("Failed to create temp directory: {}", e)))
}

/// Copy a file to a target directory, preserving the filename
pub fn copy_to_dir(source: &Path, target_dir: &Path) -> JcResult<PathBuf> {
    let filename = source
        .file_name()
        .ok_or_else(|| JcError::Other("Invalid source filename".to_string()))?;

    let dest_path = target_dir.join(filename);

    // Check if source and destination are the same file
    if source.canonicalize().ok() == dest_path.canonicalize().ok() {
        // File is already in the target directory, no need to copy
        return Ok(dest_path);
    }

    fs::copy(source, &dest_path).map_err(|e| JcError::Io(e))?;

    Ok(dest_path)
}
