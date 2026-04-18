use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::support::OsStrBytes;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunOutput {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    file: FileState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FileState {
    Missing,
    Directory,
    Bytes(Vec<u8>),
}

pub(crate) struct ReferenceContext {
    legacy_reference: PathBuf,
    legacy_runner: OsString,
    legacy_src: PathBuf,
    rust_binary: PathBuf,
}

impl ReferenceContext {
    pub(crate) fn new() -> Self {
        let legacy_reference = std::env::var_os("ENVQ_LEGACY_REFERENCE")
            .map(PathBuf::from)
            .expect("ENVQ_LEGACY_REFERENCE must point to the legacy reference checkout");
        let legacy_runner = std::env::var_os("ENVQ_LEGACY_REFERENCE_RUNNER")
            .expect("ENVQ_LEGACY_REFERENCE_RUNNER must name the legacy runner command");
        let legacy_src = legacy_reference.join("src");
        assert!(
            legacy_src.is_dir(),
            "missing legacy reference source dir: {}",
            legacy_src.display()
        );

        Self {
            legacy_reference,
            legacy_runner,
            legacy_src,
            rust_binary: PathBuf::from(env!("CARGO_BIN_EXE_envq")),
        }
    }

    pub(crate) fn run_legacy(&self, args: &[String], stdin: &[u8], env_file: &Path) -> RunOutput {
        let mut command = Command::new(&self.legacy_runner);
        command
            .arg("-m")
            .arg("envq")
            .args(expanded_args(args, env_file))
            .current_dir(&self.legacy_reference)
            .env("PYTHONPATH", &self.legacy_src)
            .env("PYTHONIOENCODING", "utf-8:surrogateescape")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        run_child(command, stdin, env_file)
    }

    pub(crate) fn run_rust(&self, args: &[String], stdin: &[u8], env_file: &Path) -> RunOutput {
        let mut command = Command::new(&self.rust_binary);
        command
            .args(expanded_args(args, env_file))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        run_child(command, stdin, env_file)
    }
}

pub(crate) fn normalize_output(output: RunOutput, path: &Path) -> RunOutput {
    RunOutput {
        code: output.code,
        stdout: normalize_path(output.stdout, path),
        stderr: normalize_path(output.stderr, path),
        file: output.file,
    }
}

fn run_child(mut command: Command, stdin: &[u8], env_file: &Path) -> RunOutput {
    let mut child = command.spawn().expect("spawn command");
    if !stdin.is_empty() {
        child
            .stdin
            .as_mut()
            .expect("child stdin")
            .write_all(stdin)
            .expect("write child stdin");
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("wait for command");
    RunOutput {
        code: output.status.code().unwrap_or(-1),
        stdout: output.stdout,
        stderr: output.stderr,
        file: file_state(env_file),
    }
}

fn expanded_args(args: &[String], env_file: &Path) -> Vec<OsString> {
    args.iter()
        .map(|arg| {
            if arg == "{path}" {
                env_file.as_os_str().to_owned()
            } else {
                OsString::from(arg)
            }
        })
        .collect()
}

fn file_state(path: &Path) -> FileState {
    if path.is_dir() {
        FileState::Directory
    } else if path.exists() {
        FileState::Bytes(fs::read(path).expect("read output file"))
    } else {
        FileState::Missing
    }
}

fn normalize_path(bytes: Vec<u8>, path: &Path) -> Vec<u8> {
    let path_bytes = path.as_os_str().encoded_bytes();
    if path_bytes.is_empty() {
        return bytes;
    }

    let mut normalized = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index..].starts_with(&path_bytes) {
            normalized.extend(b"{path}");
            index += path_bytes.len();
        } else {
            normalized.push(bytes[index]);
            index += 1;
        }
    }
    normalized
}
