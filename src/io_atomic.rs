//! Raw byte file I/O and atomic replacement helpers.

use std::fs;
#[cfg(unix)]
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use tempfile::Builder;

/// Reads a file as raw bytes.
pub fn read_bytes(path: &Path) -> io::Result<Vec<u8>> {
    fs::read(path)
}

/// Writes bytes through a same-directory temporary file and atomically replaces `path`.
pub fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let directory = containing_directory(path);
    let existing_permissions = existing_permissions(path)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("envq");
    let prefix = format!(".{file_name}.");
    let mut temporary = Builder::new()
        .prefix(&prefix)
        .suffix(".tmp")
        .tempfile_in(&directory)?;

    if let Some(permissions) = existing_permissions {
        temporary.as_file().set_permissions(permissions)?;
    }

    temporary.write_all(bytes)?;
    temporary.as_file_mut().flush()?;
    temporary.as_file().sync_all()?;

    // `persist` renames over the path itself. For symlinks, this replaces the
    // symlink with a regular file instead of following it.
    temporary
        .persist(path)
        .map_err(|error| io::Error::new(error.error.kind(), error.error))?;
    fsync_directory(&directory);
    Ok(())
}

/// Returns the directory used for same-directory temporary files.
#[must_use]
pub fn containing_directory(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

fn existing_permissions(path: &Path) -> io::Result<Option<fs::Permissions>> {
    // `metadata` follows symlinks, matching the reference behavior for mode
    // preservation before the symlink path itself is replaced.
    match fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata.permissions())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn fsync_directory(directory: &Path) {
    let Ok(file) = File::open(directory) else {
        return;
    };
    // Directory fsync is best effort; replacement success is already reported.
    let _ = file.sync_all();
}

#[cfg(not(unix))]
fn fsync_directory(_directory: &Path) {}

#[cfg(test)]
mod tests;
