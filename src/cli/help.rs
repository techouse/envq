//! Static help text and usage diagnostics.

use crate::model::ExitCode;

use super::types::{CliResult, CommandName};

/// Top-level usage prefix shared by parser errors.
pub(super) const USAGE: &str = "usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\n";

const COMMAND_USAGE: &str = "commands:
  envq [--version] [--quiet]
  get KEY PATH
  set KEY VALUE PATH [--stdout|--diff] [--check]
  set KEY - PATH [--stdout|--diff] [--check]
  clear KEY PATH [--stdout|--diff] [--check]
  unset KEY PATH [--stdout|--diff] [--check]
  remove KEY PATH [--stdout|--diff] [--check]
  has KEY PATH
  list PATH [--json|--yaml|--names] [--unique]
  completion {bash,zsh,fish,powershell,pwsh}
  help [COMMAND]
";

const REMOVE_HELP: &str =
    "usage: envq remove KEY PATH [--stdout|--diff] [--check]\n\nAlias for unset.\n";
const HELP_HELP: &str = "usage: envq help [COMMAND]\n\nPrint command-specific help.\n";

/// Returns top-level or command-specific help output.
pub(super) fn help_result(topic: Option<&str>) -> CliResult {
    let topic = match topic {
        Some(topic) => topic,
        None => {
            return CliResult {
                exit_code: ExitCode::Success,
                stdout: top_help(),
                stderr: Vec::new(),
            };
        }
    };

    let help_text = match topic {
        "get" => {
            "usage: envq get KEY PATH\n\nPrint the first matching value exactly, without adding a newline.\nReturns exit code 2 when KEY is absent.\n"
        }
        "set" => {
            "usage: envq set KEY VALUE PATH [--stdout|--diff] [--check]\n       envq set KEY - PATH [--stdout|--diff] [--check]\n\nSet KEY to VALUE, updating the first match or appending a new binding.\nUse VALUE '-' to read the value from stdin exactly.\nTrailing --stdout prints the rendered file without writing.\nTrailing --diff prints a unified diff without writing.\nTrailing --check exits 4 when the file would change and never writes.\n"
        }
        "clear" => {
            "usage: envq clear KEY PATH [--stdout|--diff] [--check]\n\nKeep or create KEY and set it to an empty value.\nSupports the same trailing output and check options as set.\n"
        }
        "unset" => {
            "usage: envq unset KEY PATH [--stdout|--diff] [--check]\n\nRemove all matching bindings for KEY.\nReturns exit code 2 when KEY is absent.\n"
        }
        "remove" => REMOVE_HELP,
        "has" => {
            "usage: envq has KEY PATH\n\nPrint nothing. Exit 0 if KEY exists, or 2 if KEY is absent.\n"
        }
        "list" => {
            "usage: envq list PATH [--json|--yaml|--names] [--unique]\n\nPrint bindings in file order as KEY<TAB>VALUE by default.\n--json prints a JSON array of {\"key\", \"value\"} objects.\n--yaml prints a dependency-free YAML sequence.\n--names prints only keys, one per line.\n--unique keeps the first binding for each key and drops later duplicates.\n"
        }
        "completion" => {
            "usage: envq completion {bash,zsh,fish,powershell,pwsh}\n\nPrint a shell completion script to stdout.\n"
        }
        "help" => HELP_HELP,
        _ => {
            return CliResult {
                exit_code: ExitCode::GeneralError,
                stdout: Vec::new(),
                stderr: format!("envq: unknown help topic: {topic}\n").into_bytes(),
            };
        }
    };

    CliResult {
        exit_code: ExitCode::Success,
        stdout: help_text.as_bytes().to_vec(),
        stderr: Vec::new(),
    }
}

/// Formats a contract-compatible operand-count usage error.
pub(super) fn command_usage_error(command_name: CommandName) -> Vec<u8> {
    format!(
        "{USAGE}envq: error: expected operands: envq {}\n",
        command_signature(command_name)
    )
    .into_bytes()
}

/// Returns top-level help output.
pub(super) fn top_help() -> Vec<u8> {
    format!(
        "{USAGE}
Read and edit .env files deterministically.

positional arguments:
  command     command to run
  ...         command operands

options:
  -h, --help  show this help message and exit
  --version   show program's version number and exit
  --quiet     suppress command diagnostics on stderr

{COMMAND_USAGE}"
    )
    .into_bytes()
}

fn command_signature(command_name: CommandName) -> &'static str {
    match command_name {
        CommandName::Get => "get KEY PATH",
        CommandName::Set => "set KEY VALUE PATH",
        CommandName::Clear => "clear KEY PATH",
        CommandName::Unset => "unset KEY PATH",
        CommandName::Remove => "remove KEY PATH",
        CommandName::Has => "has KEY PATH",
        CommandName::List => "list PATH",
        CommandName::Completion => "completion {bash,zsh,fish,powershell,pwsh}",
        CommandName::Help => "help [COMMAND]",
    }
}
