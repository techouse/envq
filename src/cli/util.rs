use std::ffi::OsStr;
use std::io;
use std::path::Path;

/// Converts a key operand to a string for later key validation.
pub(super) fn key_operand(argument: &OsStr) -> String {
    match std::str::from_utf8(&os_bytes(argument)) {
        Ok(value) => value.to_owned(),
        Err(_error) => os_string_lossy(argument),
    }
}

/// Formats a missing-key diagnostic.
pub(super) fn key_not_found_message(key: &str) -> Vec<u8> {
    format!("envq: {key}: key not found\n").into_bytes()
}

/// Formats a path-related OS diagnostic with normalized wording for common errors.
pub(super) fn os_error_message(path: &Path, error: &io::Error) -> Vec<u8> {
    format!("envq: {}: {}\n", path.display(), os_error_detail(error)).into_bytes()
}

/// Formats a stdin read diagnostic.
pub(super) fn stdin_error_message(error: &io::Error) -> Vec<u8> {
    format!("envq: stdin: {}\n", os_error_detail(error)).into_bytes()
}

fn os_error_detail(error: &io::Error) -> String {
    match error.kind() {
        io::ErrorKind::NotFound => "No such file or directory".to_owned(),
        io::ErrorKind::PermissionDenied => "Permission denied".to_owned(),
        io::ErrorKind::AlreadyExists => "File exists".to_owned(),
        io::ErrorKind::IsADirectory => "Is a directory".to_owned(),
        _ => error.to_string(),
    }
}

#[cfg(unix)]
pub(super) fn os_bytes(value: &OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;

    value.as_bytes().to_vec()
}

#[cfg(not(unix))]
pub(super) fn os_bytes(value: &OsStr) -> Vec<u8> {
    value.to_string_lossy().as_bytes().to_vec()
}

pub(super) fn os_string_lossy(value: &OsStr) -> String {
    String::from_utf8_lossy(&os_bytes(value)).into_owned()
}
