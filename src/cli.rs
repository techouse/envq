//! CLI adapter that preserves raw stdout and stderr bytes.

mod completion;
mod execute;
mod help;
mod listing;
mod parse;
mod spec;
mod tail;
mod types;
mod util;

use std::ffi::OsString;
use std::io::Read;

use crate::model::ExitCode;

pub use types::CliResult;

use self::execute::execute;
use self::parse::parse_command;
use self::types::ParseControl;

/// Runs the envq CLI with already-split arguments and an explicit stdin stream.
///
/// `argv` must not include the executable name. The returned stdout and stderr
/// are raw bytes so invalid UTF-8 can be reported compatibly.
pub fn run<I, S>(argv: I, stdin: &mut dyn Read) -> CliResult
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let argv = argv.into_iter().map(Into::into).collect::<Vec<_>>();
    match parse_command(&argv) {
        Ok(parsed) => {
            let mut result = execute(&parsed.command, stdin);
            if parsed.global_options.quiet && result.exit_code != ExitCode::Success {
                result.stderr.clear();
            }
            result
        }
        Err(ParseControl::Help(help)) => CliResult {
            exit_code: ExitCode::Success,
            stdout: help.message,
            stderr: Vec::new(),
        },
        Err(ParseControl::Usage(error)) => CliResult {
            exit_code: ExitCode::GeneralError,
            stdout: Vec::new(),
            stderr: error.message,
        },
    }
}

#[cfg(test)]
mod tests;
