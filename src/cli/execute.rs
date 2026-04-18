//! Command execution after arguments have been parsed.

use std::io::{self, Read};
use std::path::Path;

use crate::diagnostics::{invalid_key_message, is_valid_key};
use crate::diff::unified_diff;
use crate::editor::{get_value, has_key, list_bindings, set_value, unset_key};
use crate::io_atomic::{containing_directory, read_bytes, write_bytes_atomic};
use crate::model::{Document, ExitCode};
use crate::parser::parse_document;
use crate::render::render_document;

use super::completion::completion_script;
use super::help::help_result;
use super::listing::list_stdout;
use super::types::{CliResult, Command, ListOptions, MutationMode, MutationOptions};
use super::util::{key_not_found_message, os_error_message, stdin_error_message};

#[derive(Clone, Copy)]
enum DocumentCommand<'a> {
    Get {
        key: &'a str,
        path: &'a Path,
    },
    Set {
        key: &'a str,
        value: &'a [u8],
        path: &'a Path,
        options: MutationOptions,
    },
    Clear {
        key: &'a str,
        path: &'a Path,
        options: MutationOptions,
    },
    Unset {
        key: &'a str,
        path: &'a Path,
        options: MutationOptions,
    },
    Has {
        key: &'a str,
        path: &'a Path,
    },
    List {
        path: &'a Path,
        options: ListOptions,
    },
}

struct LoadedDocument {
    source_text: Vec<u8>,
    document: Document,
}

/// Executes a parsed command and returns raw stdout/stderr bytes.
pub(super) fn execute(command: &Command, stdin: &mut dyn Read) -> CliResult {
    match command {
        Command::Completion { shell } => CliResult {
            exit_code: ExitCode::Success,
            stdout: completion_script(*shell),
            stderr: Vec::new(),
        },
        Command::Help { topic } => help_result(topic.as_deref()),
        Command::Get { key, path } => execute_document_command(
            DocumentCommand::Get {
                key,
                path: path.as_path(),
            },
            stdin,
        ),
        Command::Set {
            key,
            value,
            path,
            options,
        } => execute_document_command(
            DocumentCommand::Set {
                key,
                value,
                path: path.as_path(),
                options: *options,
            },
            stdin,
        ),
        Command::Clear { key, path, options } => execute_document_command(
            DocumentCommand::Clear {
                key,
                path: path.as_path(),
                options: *options,
            },
            stdin,
        ),
        Command::Unset { key, path, options } => execute_document_command(
            DocumentCommand::Unset {
                key,
                path: path.as_path(),
                options: *options,
            },
            stdin,
        ),
        Command::Has { key, path } => execute_document_command(
            DocumentCommand::Has {
                key,
                path: path.as_path(),
            },
            stdin,
        ),
        Command::List { path, options } => execute_document_command(
            DocumentCommand::List {
                path: path.as_path(),
                options: *options,
            },
            stdin,
        ),
    }
}

fn execute_document_command(command: DocumentCommand<'_>, stdin: &mut dyn Read) -> CliResult {
    if let Some(invalid_key) = invalid_command_key(command) {
        return CliResult {
            exit_code: ExitCode::ValidationError,
            stdout: Vec::new(),
            stderr: invalid_key_message(invalid_key),
        };
    }

    let loaded_document = match read_document_for_command(command) {
        Ok(loaded_document) => loaded_document,
        Err(result) => return result,
    };
    execute_loaded_document_command(
        command,
        &loaded_document.source_text,
        &loaded_document.document,
        stdin,
    )
}

fn execute_loaded_document_command(
    command: DocumentCommand<'_>,
    source_text: &[u8],
    document: &Document,
    stdin: &mut dyn Read,
) -> CliResult {
    match command {
        DocumentCommand::Get { key, .. } => match get_value(document, key) {
            Some(value) => CliResult {
                exit_code: ExitCode::Success,
                stdout: value.to_vec(),
                stderr: Vec::new(),
            },
            None => CliResult {
                exit_code: ExitCode::KeyNotFound,
                stdout: Vec::new(),
                stderr: key_not_found_message(key),
            },
        },
        DocumentCommand::Has { key, .. } => {
            if has_key(document, key) {
                CliResult::success()
            } else {
                CliResult {
                    exit_code: ExitCode::KeyNotFound,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                }
            }
        }
        DocumentCommand::List { options, .. } => CliResult {
            exit_code: ExitCode::Success,
            stdout: list_stdout(&list_bindings(document), options),
            stderr: Vec::new(),
        },
        DocumentCommand::Set {
            key,
            value,
            path,
            options,
        } => {
            let value_result = set_command_value(value, stdin);
            let value = match value_result {
                Ok(value) => value,
                Err(result) => return result,
            };
            let updated_document = set_value(document, key, &value);
            finish_mutation(path, source_text, &updated_document, options)
        }
        DocumentCommand::Clear { key, path, options } => {
            let updated_document = set_value(document, key, b"");
            finish_mutation(path, source_text, &updated_document, options)
        }
        DocumentCommand::Unset { key, path, options } => {
            let unset_result = unset_key(document, key);
            if unset_result.removed_count == 0 {
                return CliResult {
                    exit_code: ExitCode::KeyNotFound,
                    stdout: Vec::new(),
                    stderr: key_not_found_message(key),
                };
            }
            finish_mutation(path, source_text, &unset_result.document, options)
        }
    }
}

fn invalid_command_key(command: DocumentCommand<'_>) -> Option<&str> {
    match command {
        DocumentCommand::Get { key, .. } => invalid_key(key),
        DocumentCommand::Set { key, .. } => invalid_key(key),
        DocumentCommand::Clear { key, .. } => invalid_key(key),
        DocumentCommand::Unset { key, .. } => invalid_key(key),
        DocumentCommand::Has { key, .. } => invalid_key(key),
        DocumentCommand::List { .. } => None,
    }
}

fn invalid_key(key: &str) -> Option<&str> {
    match is_valid_key(key) {
        true => None,
        false => Some(key),
    }
}

fn read_document_for_command(command: DocumentCommand<'_>) -> Result<LoadedDocument, CliResult> {
    match command {
        DocumentCommand::Set { path, .. } => read_document_for_set(path),
        DocumentCommand::Clear { path, .. } => read_document_for_set(path),
        DocumentCommand::Get { path, .. } => read_document(path),
        DocumentCommand::Unset { path, .. } => read_document(path),
        DocumentCommand::Has { path, .. } => read_document(path),
        DocumentCommand::List { path, .. } => read_document(path),
    }
}

fn read_document(path: &Path) -> Result<LoadedDocument, CliResult> {
    match read_bytes(path) {
        Ok(source_text) => {
            let document = parse_document(&source_text);
            Ok(LoadedDocument {
                source_text,
                document,
            })
        }
        Err(error) => Err(CliResult {
            exit_code: ExitCode::GeneralError,
            stdout: Vec::new(),
            stderr: os_error_message(path, &error),
        }),
    }
}

fn read_document_for_set(path: &Path) -> Result<LoadedDocument, CliResult> {
    match read_bytes(path) {
        Ok(source_text) => {
            let document = parse_document(&source_text);
            Ok(LoadedDocument {
                source_text,
                document,
            })
        }
        Err(error) => handle_set_read_error(path, error),
    }
}

fn handle_set_read_error(path: &Path, error: io::Error) -> Result<LoadedDocument, CliResult> {
    if error.kind() != io::ErrorKind::NotFound {
        return Err(CliResult {
            exit_code: ExitCode::GeneralError,
            stdout: Vec::new(),
            stderr: os_error_message(path, &error),
        });
    }

    // `set` and `clear` may create a missing file, but only when the
    // parent directory already exists.
    if containing_directory(path).is_dir() {
        Ok(LoadedDocument {
            source_text: Vec::new(),
            document: parse_document(b""),
        })
    } else {
        Err(CliResult {
            exit_code: ExitCode::GeneralError,
            stdout: Vec::new(),
            stderr: os_error_message(path, &error),
        })
    }
}

fn set_command_value(value: &[u8], stdin: &mut dyn Read) -> Result<Vec<u8>, CliResult> {
    if value != b"-" {
        return Ok(value.to_vec());
    }

    // Stdin is the value exactly; trailing newlines are data.
    let mut stdin_value = Vec::new();
    match stdin.read_to_end(&mut stdin_value) {
        Ok(_bytes_read) => Ok(stdin_value),
        Err(error) => Err(CliResult {
            exit_code: ExitCode::GeneralError,
            stdout: Vec::new(),
            stderr: stdin_error_message(&error),
        }),
    }
}

pub(super) fn finish_mutation(
    path: &Path,
    source_text: &[u8],
    document: &Document,
    options: MutationOptions,
) -> CliResult {
    let rendered_text = render_document(document);

    if options.check {
        // Check mode never writes. Output mode only controls what, if anything,
        // accompanies the would-change exit code.
        let stdout = match options.mode {
            MutationMode::Stdout => rendered_text.clone(),
            MutationMode::Diff => unified_diff(path, source_text, &rendered_text),
            MutationMode::Write => Vec::new(),
        };
        let exit_code = if rendered_text == source_text {
            ExitCode::Success
        } else {
            ExitCode::WouldChange
        };
        return CliResult {
            exit_code,
            stdout,
            stderr: Vec::new(),
        };
    }

    match options.mode {
        MutationMode::Stdout => CliResult {
            exit_code: ExitCode::Success,
            stdout: rendered_text,
            stderr: Vec::new(),
        },
        MutationMode::Diff => CliResult {
            exit_code: ExitCode::Success,
            stdout: unified_diff(path, source_text, &rendered_text),
            stderr: Vec::new(),
        },
        MutationMode::Write => {
            if rendered_text == source_text {
                return CliResult::success();
            }
            match write_bytes_atomic(path, &rendered_text) {
                Ok(()) => CliResult::success(),
                Err(error) => CliResult {
                    exit_code: ExitCode::GeneralError,
                    stdout: Vec::new(),
                    stderr: os_error_message(path, &error),
                },
            }
        }
    }
}
