//! Clap command metadata used for validation and drift guards.
//!
//! User-visible help and diagnostics are still emitted by compatibility code
//! instead of raw clap output.

use clap::builder::PossibleValuesParser;
use clap::{Arg, ArgAction, ArgGroup, Command as ClapCommand, ValueHint};

use super::types::{COMMAND_NAMES, CommandName};

pub(super) const COMPLETION_SHELLS: &[&str] = &["bash", "zsh", "fish", "powershell", "pwsh"];
#[cfg(test)]
pub(super) const MUTATION_OUTPUT_OPTIONS: &[&str] = &["stdout", "diff"];
#[cfg(test)]
pub(super) const LIST_FORMAT_OPTIONS: &[&str] = &["json", "yaml", "names"];

/// Full command surface used by tests to detect drift from documented commands.
#[allow(dead_code)]
pub(super) fn command_spec() -> ClapCommand {
    ClapCommand::new("envq")
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("quiet").long("quiet").action(ArgAction::SetTrue))
        .subcommand(command_with_positionals(CommandName::Get, ["KEY", "PATH"]))
        .subcommand(mutation_command_with_positionals(
            CommandName::Set,
            ["KEY", "VALUE", "PATH"],
        ))
        .subcommand(mutation_command_with_positionals(
            CommandName::Clear,
            ["KEY", "PATH"],
        ))
        .subcommand(mutation_command_with_positionals(
            CommandName::Unset,
            ["KEY", "PATH"],
        ))
        .subcommand(mutation_command_with_positionals(
            CommandName::Remove,
            ["KEY", "PATH"],
        ))
        .subcommand(command_with_positionals(CommandName::Has, ["KEY", "PATH"]))
        .subcommand(list_command_with_positionals())
        .subcommand(
            subcommand(CommandName::Completion).arg(
                Arg::new("SHELL")
                    .required(true)
                    .value_parser(PossibleValuesParser::new(COMPLETION_SHELLS)),
            ),
        )
        .subcommand(
            subcommand(CommandName::Help)
                .arg(Arg::new("COMMAND").value_parser(PossibleValuesParser::new(COMMAND_NAMES))),
        )
}

/// Clap parser for options that appear after mutation operands.
pub(super) fn mutation_tail_spec(command_name: CommandName) -> ClapCommand {
    add_mutation_options(
        ClapCommand::new(command_name.as_str())
            .no_binary_name(true)
            .disable_help_flag(true)
            .disable_version_flag(true),
    )
}

/// Clap parser for options that appear after the list path operand.
pub(super) fn list_tail_spec() -> ClapCommand {
    add_list_options(
        ClapCommand::new(CommandName::List.as_str())
            .no_binary_name(true)
            .disable_help_flag(true)
            .disable_version_flag(true),
    )
}

/// Clap parser for completion shell validation.
pub(super) fn completion_shell_spec() -> ClapCommand {
    ClapCommand::new(CommandName::Completion.as_str())
        .no_binary_name(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("shell")
                .required(true)
                .value_parser(PossibleValuesParser::new(COMPLETION_SHELLS)),
        )
}

fn command_with_positionals<const N: usize>(
    command_name: CommandName,
    positionals: [&'static str; N],
) -> ClapCommand {
    positionals
        .into_iter()
        .fold(subcommand(command_name), |command, name| {
            command.arg(positional_arg(name).required(true))
        })
}

fn mutation_command_with_positionals<const N: usize>(
    command_name: CommandName,
    positionals: [&'static str; N],
) -> ClapCommand {
    add_mutation_options(command_with_positionals(command_name, positionals))
}

fn list_command_with_positionals() -> ClapCommand {
    add_list_options(command_with_positionals(CommandName::List, ["PATH"]))
}

fn subcommand(command_name: CommandName) -> ClapCommand {
    ClapCommand::new(command_name.as_str())
        .disable_help_flag(true)
        .disable_version_flag(true)
}

fn positional_arg(name: &'static str) -> Arg {
    let argument = Arg::new(name);
    if name == "PATH" {
        argument.value_hint(ValueHint::FilePath)
    } else {
        argument
    }
}

fn add_mutation_options(command: ClapCommand) -> ClapCommand {
    command
        .arg(
            Arg::new("stdout")
                .long("stdout")
                .action(ArgAction::SetTrue)
                .group("output"),
        )
        .arg(
            Arg::new("diff")
                .long("diff")
                .action(ArgAction::SetTrue)
                .group("output"),
        )
        .arg(Arg::new("check").long("check").action(ArgAction::SetTrue))
        .group(ArgGroup::new("output").multiple(false))
}

fn add_list_options(command: ClapCommand) -> ClapCommand {
    command
        .arg(
            Arg::new("json")
                .long("json")
                .action(ArgAction::SetTrue)
                .group("format"),
        )
        .arg(
            Arg::new("yaml")
                .long("yaml")
                .action(ArgAction::SetTrue)
                .group("format"),
        )
        .arg(
            Arg::new("names")
                .long("names")
                .action(ArgAction::SetTrue)
                .group("format"),
        )
        .arg(Arg::new("unique").long("unique").action(ArgAction::SetTrue))
        .group(ArgGroup::new("format").multiple(false))
}
