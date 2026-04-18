use std::ffi::OsString;
use std::fs;
use std::io::{self, Cursor, Read};

use crate::model::{Document, ExitCode};

use super::spec::{LIST_FORMAT_OPTIONS, MUTATION_OUTPUT_OPTIONS};
use super::types::{
    COMMAND_NAMES, CommandName, CompletionShell, ListOptions, ListOutputFormat, MutationMode,
    MutationOptions,
};
use super::{CliResult, run, spec};

struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "stdin failed"))
    }
}

fn default_newline() -> &'static [u8] {
    if cfg!(windows) { b"\r\n" } else { b"\n" }
}

fn bytes_with_default_newline(text: &[u8]) -> Vec<u8> {
    let mut bytes = text.to_vec();
    bytes.extend(default_newline());
    bytes
}

fn assert_directory_like_error(stderr: &[u8]) {
    let message = String::from_utf8_lossy(stderr);
    assert!(
        stderr.ends_with(b": Is a directory\n")
            || stderr.ends_with(b": Permission denied\n")
            || stderr.ends_with(b": File exists\n")
            || message.contains("Access is denied")
            || message.contains("The parameter is incorrect"),
        "{}",
        message
    );
}

mod completion;
mod diff;
mod document_commands;
mod errors;
mod list;
mod parse;
