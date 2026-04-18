use std::ffi::OsString;
use std::path::PathBuf;

use crate::model::ExitCode;

/// Command names accepted by the top-level parser, in help order.
pub(super) const COMMAND_NAMES: &[&str] = &[
    "get",
    "set",
    "clear",
    "unset",
    "remove",
    "has",
    "list",
    "completion",
    "help",
];

/// Commands that accept mutation tail options.
pub(super) const MUTATING_COMMAND_NAMES: &[CommandName] = &[
    CommandName::Set,
    CommandName::Clear,
    CommandName::Unset,
    CommandName::Remove,
];

/// Result produced by the library CLI entry point.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliResult {
    /// Process exit status.
    pub exit_code: ExitCode,
    /// Raw stdout bytes.
    pub stdout: Vec<u8>,
    /// Raw stderr bytes.
    pub stderr: Vec<u8>,
}

impl CliResult {
    /// Empty successful result.
    #[must_use]
    pub fn success() -> Self {
        Self {
            exit_code: ExitCode::Success,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CommandName {
    Get,
    Set,
    Clear,
    Unset,
    Remove,
    Has,
    List,
    Completion,
    Help,
}

impl CommandName {
    /// Parses a command name after global options have been consumed.
    pub(super) fn from_str(value: &str) -> Option<Self> {
        match value {
            "get" => Some(Self::Get),
            "set" => Some(Self::Set),
            "clear" => Some(Self::Clear),
            "unset" => Some(Self::Unset),
            "remove" => Some(Self::Remove),
            "has" => Some(Self::Has),
            "list" => Some(Self::List),
            "completion" => Some(Self::Completion),
            "help" => Some(Self::Help),
            _ => None,
        }
    }

    /// Static command spelling used in diagnostics and specs.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Set => "set",
            Self::Clear => "clear",
            Self::Unset => "unset",
            Self::Remove => "remove",
            Self::Has => "has",
            Self::List => "list",
            Self::Completion => "completion",
            Self::Help => "help",
        }
    }

    /// Required operand count before any command-specific tail options.
    pub(super) const fn operand_count(self) -> usize {
        match self {
            Self::Get => 2,
            Self::Set => 3,
            Self::Clear => 2,
            Self::Unset | Self::Remove => 2,
            Self::Has => 2,
            Self::List => 1,
            Self::Completion => 1,
            Self::Help => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MutationMode {
    Write,
    Stdout,
    Diff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ListOutputFormat {
    Table,
    Json,
    Yaml,
    Names,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MutationOptions {
    pub(super) mode: MutationMode,
    pub(super) check: bool,
}

impl Default for MutationOptions {
    fn default() -> Self {
        Self {
            mode: MutationMode::Write,
            check: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ListOptions {
    pub(super) output_format: ListOutputFormat,
    pub(super) unique: bool,
}

impl Default for ListOptions {
    fn default() -> Self {
        Self {
            output_format: ListOutputFormat::Table,
            unique: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct GlobalOptions {
    pub(super) quiet: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Command {
    Get {
        key: String,
        path: PathBuf,
    },
    Set {
        key: String,
        value: Vec<u8>,
        path: PathBuf,
        options: MutationOptions,
    },
    Clear {
        key: String,
        path: PathBuf,
        options: MutationOptions,
    },
    Unset {
        key: String,
        path: PathBuf,
        options: MutationOptions,
    },
    Has {
        key: String,
        path: PathBuf,
    },
    List {
        path: PathBuf,
        options: ListOptions,
    },
    Completion {
        shell: CompletionShell,
    },
    Help {
        topic: Option<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PreparedOperands {
    pub(super) operands: Vec<OsString>,
    pub(super) mutation_options: MutationOptions,
    pub(super) list_options: ListOptions,
}

impl PreparedOperands {
    pub(super) fn new(operands: Vec<OsString>) -> Self {
        Self {
            operands,
            mutation_options: MutationOptions::default(),
            list_options: ListOptions::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ParsedCommand {
    pub(super) command: Command,
    pub(super) global_options: GlobalOptions,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct UsageError {
    pub(super) message: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct HelpRequested {
    pub(super) message: Vec<u8>,
}

pub(super) enum ParseControl {
    Help(HelpRequested),
    Usage(UsageError),
}
