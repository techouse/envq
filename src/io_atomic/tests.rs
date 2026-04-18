#[cfg(unix)]
use std::fs;

use super::{containing_directory, read_bytes, write_bytes_atomic};

#[test]
fn writes_and_reads_bytes() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let target = directory.path().join(".env");

    write_bytes_atomic(&target, b"RAW=\xff\n").expect("write bytes");

    assert_eq!(read_bytes(&target).expect("read bytes"), b"RAW=\xff\n");
}

#[test]
fn containing_directory_defaults_to_current_directory_for_bare_paths() {
    assert_eq!(
        containing_directory(std::path::Path::new(".env")),
        std::path::Path::new(".")
    );
}

#[cfg(unix)]
#[test]
fn preserves_mode_bits() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().expect("create tempdir");
    let target = directory.path().join(".env");
    fs::write(&target, b"A=1\n").expect("write source");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).expect("chmod source");

    write_bytes_atomic(&target, b"A=2\n").expect("write bytes");

    let mode = fs::metadata(&target)
        .expect("metadata")
        .permissions()
        .mode()
        & 0o7777;
    assert_eq!(mode, 0o640);
}

#[cfg(unix)]
#[test]
fn replaces_symlink_itself() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().expect("create tempdir");
    let target = directory.path().join("target.env");
    let link = directory.path().join("link.env");
    fs::write(&target, b"A=1\n").expect("write target");
    symlink(&target, &link).expect("create symlink");

    write_bytes_atomic(&link, b"A=2\n").expect("write link path");

    assert!(
        !fs::symlink_metadata(&link)
            .expect("link metadata")
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read(&link).expect("read link replacement"), b"A=2\n");
    assert_eq!(fs::read(&target).expect("read original target"), b"A=1\n");
}

#[cfg(unix)]
#[test]
fn fsync_directory_ignores_open_errors() {
    super::fsync_directory(std::path::Path::new("/definitely/missing/envq-directory"));
}

#[cfg(unix)]
#[test]
fn existing_permissions_surfaces_metadata_errors() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().expect("create tempdir");
    let loop_link = directory.path().join("loop.env");
    symlink(&loop_link, &loop_link).expect("create symlink loop");

    let error = super::existing_permissions(&loop_link).expect_err("metadata should fail");
    assert_ne!(error.kind(), std::io::ErrorKind::NotFound);
}
