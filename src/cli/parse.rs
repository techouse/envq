use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use super::completion::completion_shell;
use super::help::{USAGE, command_usage_error, top_help};
use super::tail::{prepared_list_operands, prepared_mutation_operands};
use super::types::{
    COMMAND_NAMES, Command, CommandName, GlobalOptions, HelpRequested, MUTATING_COMMAND_NAMES,
    ParseControl, ParsedCommand, PreparedOperands, UsageError,
};
use super::util::{key_operand, os_bytes, os_string_lossy};

/// Parses global options, command name, operands, and compatibility-preserving tails.
pub(super) fn parse_command(argv: &[OsString]) -> Result<ParsedCommand, ParseControl> {
    if argv.is_empty() {
        return Err(ParseControl::Help(HelpRequested {
            message: top_help(),
        }));
    }

    let mut index = 0;
    let mut global_options = GlobalOptions::default();
    // Global flags are accepted only before the command for compatibility.
    while index < argv.len() {
        let Some(argument) = argv[index].to_str() else {
            break;
        };
        match argument {
            "--quiet" => {
                global_options.quiet = true;
                index += 1;
            }
            "--version" => {
                return Err(ParseControl::Help(HelpRequested {
                    message: format!("envq {}\n", env!("CARGO_PKG_VERSION")).into_bytes(),
                }));
            }
            "--help" | "-h" => {
                return Err(ParseControl::Help(HelpRequested {
                    message: top_help(),
                }));
            }
            _ => break,
        }
    }

    if index >= argv.len() {
        return Err(ParseControl::Help(HelpRequested {
            message: top_help(),
        }));
    }

    let command_name = namespace_command(&argv[index])?;
    let operands = argv[index + 1..].to_vec();
    let prepared_operands = prepared_operands(command_name, operands)?;

    if !has_expected_operand_count(command_name, &prepared_operands.operands) {
        return Err(ParseControl::Usage(UsageError {
            message: command_usage_error(command_name),
        }));
    }

    Ok(ParsedCommand {
        command: make_command(command_name, prepared_operands)?,
        global_options,
    })
}

fn make_command(
    command_name: CommandName,
    prepared_operands: PreparedOperands,
) -> Result<Command, ParseControl> {
    let operands = prepared_operands.operands;
    match command_name {
        CommandName::Get => Ok(Command::Get {
            key: key_operand(&operands[0]),
            path: PathBuf::from(&operands[1]),
        }),
        CommandName::Set => Ok(Command::Set {
            key: key_operand(&operands[0]),
            value: os_bytes(&operands[1]),
            path: PathBuf::from(&operands[2]),
            options: prepared_operands.mutation_options,
        }),
        CommandName::Clear => Ok(Command::Clear {
            key: key_operand(&operands[0]),
            path: PathBuf::from(&operands[1]),
            options: prepared_operands.mutation_options,
        }),
        CommandName::Unset | CommandName::Remove => Ok(Command::Unset {
            key: key_operand(&operands[0]),
            path: PathBuf::from(&operands[1]),
            options: prepared_operands.mutation_options,
        }),
        CommandName::Has => Ok(Command::Has {
            key: key_operand(&operands[0]),
            path: PathBuf::from(&operands[1]),
        }),
        CommandName::List => Ok(Command::List {
            path: PathBuf::from(&operands[0]),
            options: prepared_operands.list_options,
        }),
        CommandName::Completion => Ok(Command::Completion {
            shell: completion_shell(&operands[0])?,
        }),
        CommandName::Help => Ok(Command::Help {
            topic: operands.first().map(|operand| os_string_lossy(operand)),
        }),
    }
}

fn prepared_operands(
    command_name: CommandName,
    operands: Vec<OsString>,
) -> Result<PreparedOperands, ParseControl> {
    // Tail options are parsed after required operands so option-looking values
    // remain valid operands, for example `envq set KEY --stdout PATH`.
    if MUTATING_COMMAND_NAMES.contains(&command_name) {
        return prepared_mutation_operands(command_name, operands);
    }
    if command_name == CommandName::List {
        return prepared_list_operands(command_name, operands);
    }
    Ok(PreparedOperands::new(operands))
}

fn namespace_command(argument: &OsStr) -> Result<CommandName, ParseControl> {
    if let Some(value) = argument.to_str() {
        if let Some(command_name) = CommandName::from_str(value) {
            return Ok(command_name);
        }
        let choices = COMMAND_NAMES.join(", ");
        return Err(ParseControl::Usage(UsageError {
            message: format!(
                "{USAGE}envq: error: argument command: invalid choice: '{value}' (choose from {choices})\n"
            )
            .into_bytes(),
        }));
    }

    Err(ParseControl::Usage(UsageError {
        message: format!(
            "{USAGE}envq: error: argument command: invalid choice: {:?}\n",
            os_string_lossy(argument)
        )
        .into_bytes(),
    }))
}

fn has_expected_operand_count(command_name: CommandName, operands: &[OsString]) -> bool {
    if command_name == CommandName::Help {
        operands.len() <= 1
    } else {
        operands.len() == command_name.operand_count()
    }
}
