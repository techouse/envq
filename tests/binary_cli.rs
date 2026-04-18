#[cfg(windows)]
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn binary_wrapper_writes_stdout_and_exits_successfully() {
    let output = envq_command()
        .arg("--version")
        .output()
        .expect("run envq binary");

    assert!(output.status.success());
    assert!(output.stdout.starts_with(b"envq "));
    assert!(output.stderr.is_empty());
}

#[test]
fn binary_wrapper_writes_stderr_and_preserves_exit_code() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    std::fs::write(&env_file, b"A=1\n").expect("write env file");

    let output = envq_command()
        .arg("get")
        .arg("B")
        .arg(&env_file)
        .output()
        .expect("run envq binary");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"envq: B: key not found\n");
}

#[test]
fn binary_wrapper_preserves_invalid_utf8_stdout_bytes() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    std::fs::write(&env_file, b"RAW=\xff\n").expect("write env file");

    let output = envq_command()
        .arg("get")
        .arg("RAW")
        .arg(&env_file)
        .output()
        .expect("run envq binary");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"\xff");
    assert!(output.stderr.is_empty());
}

#[test]
fn binary_wrapper_handles_paths_with_spaces_and_unicode() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let nested = directory.path().join("space dir").join("unicode-\u{00e9}");
    std::fs::create_dir_all(&nested).expect("create nested path");
    let env_file = nested.join("settings.env");

    let set = envq_command()
        .arg("set")
        .arg("API_KEY")
        .arg("two words")
        .arg(&env_file)
        .output()
        .expect("run envq set");
    assert!(set.status.success());
    assert!(set.stderr.is_empty());

    let get = envq_command()
        .arg("get")
        .arg("API_KEY")
        .arg(&env_file)
        .output()
        .expect("run envq get");
    assert!(get.status.success());
    assert_eq!(get.stdout, b"two words");
    assert!(get.stderr.is_empty());
}

#[cfg(windows)]
#[test]
fn binary_wrapper_handles_windows_extended_length_paths() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let mut long_dir = extended_length_path(directory.path());
    for index in 0..10 {
        long_dir.push(format!("segment-{index:02}-abcdefghijklmnopqrstuvwxyz"));
    }
    std::fs::create_dir_all(&long_dir).expect("create long path");
    let env_file = long_dir.join("values.env");

    let set = envq_command()
        .arg("set")
        .arg("LONG_PATH")
        .arg("ok")
        .arg(&env_file)
        .output()
        .expect("run envq set");
    assert!(set.status.success());
    assert!(set.stderr.is_empty());

    let get = envq_command()
        .arg("get")
        .arg("LONG_PATH")
        .arg(&env_file)
        .output()
        .expect("run envq get");
    assert!(get.status.success());
    assert_eq!(get.stdout, b"ok");
    assert!(get.stderr.is_empty());
}

#[test]
fn binary_wrapper_updates_only_one_binding_in_large_file() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    let mut source = Vec::new();
    let mut expected = Vec::new();
    for index in 0..10_000 {
        source.extend(format!("KEY_{index}=same\n").as_bytes());
        if index == 9_000 {
            expected.extend(format!("KEY_{index}=changed\n").as_bytes());
        } else {
            expected.extend(format!("KEY_{index}=same\n").as_bytes());
        }
    }
    std::fs::write(&env_file, source).expect("write env file");

    let output = envq_command()
        .arg("set")
        .arg("KEY_9000")
        .arg("changed")
        .arg(&env_file)
        .output()
        .expect("run envq set");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert_eq!(std::fs::read(&env_file).expect("read env file"), expected);
}

#[test]
fn binary_wrapper_set_is_idempotent() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    std::fs::write(&env_file, b"A=1\nB=2\n").expect("write env file");

    let first = envq_command()
        .arg("set")
        .arg("A")
        .arg("2")
        .arg(&env_file)
        .output()
        .expect("run first envq set");
    assert!(first.status.success());
    assert!(first.stdout.is_empty());
    assert!(first.stderr.is_empty());
    let after_first = std::fs::read(&env_file).expect("read env file");

    let second = envq_command()
        .arg("set")
        .arg("A")
        .arg("2")
        .arg(&env_file)
        .arg("--check")
        .output()
        .expect("run second envq set");
    assert!(second.status.success());
    assert!(second.stdout.is_empty());
    assert!(second.stderr.is_empty());
    assert_eq!(
        std::fs::read(&env_file).expect("read env file"),
        after_first
    );
}

fn envq_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_envq"))
}

#[cfg(windows)]
fn extended_length_path(path: &Path) -> PathBuf {
    let absolute = path
        .canonicalize()
        .unwrap_or_else(|_error| canonicalize_missing_path(path));
    let text = absolute.as_os_str().to_string_lossy();
    if text.starts_with(r"\\?\") {
        absolute
    } else if let Some(unc_path) = text.strip_prefix(r"\\") {
        PathBuf::from(format!(r"\\?\UNC\{unc_path}"))
    } else {
        PathBuf::from(format!(r"\\?\{text}"))
    }
}

#[cfg(windows)]
fn canonicalize_missing_path(path: &Path) -> PathBuf {
    let parent = path
        .parent()
        .expect("path has parent")
        .canonicalize()
        .expect("canonicalize parent");
    match path.file_name() {
        Some(file_name) => parent.join(file_name),
        None => parent,
    }
}
