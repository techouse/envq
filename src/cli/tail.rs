use std::ffi::OsString;

use super::help::{USAGE, command_usage_error};
use super::spec;
use super::types::{
    CommandName, ListOptions, ListOutputFormat, MutationMode, MutationOptions, ParseControl,
    PreparedOperands, UsageError,
};

/// Parses trailing mutation options after required operands have been sliced off.
pub(super) fn prepared_mutation_operands(
    command_name: CommandName,
    operands: Vec<OsString>,
) -> Result<PreparedOperands, ParseControl> {
    let required_operands = command_name.operand_count();
    if operands.len() <= required_operands {
        return Ok(PreparedOperands::new(operands));
    }

    let tail = &operands[required_operands..];
    let matches = spec::mutation_tail_spec(command_name)
        .try_get_matches_from(tail.iter().cloned())
        .map_err(|_error| mutation_tail_error(command_name, tail))?;

    let mode = if matches.get_flag("stdout") {
        MutationMode::Stdout
    } else if matches.get_flag("diff") {
        MutationMode::Diff
    } else {
        MutationMode::Write
    };
    let check = matches.get_flag("check");

    Ok(PreparedOperands {
        operands: operands[..required_operands].to_vec(),
        mutation_options: MutationOptions { mode, check },
        list_options: ListOptions::default(),
    })
}

/// Parses trailing list options after the required path operand.
pub(super) fn prepared_list_operands(
    command_name: CommandName,
    operands: Vec<OsString>,
) -> Result<PreparedOperands, ParseControl> {
    let required_operands = command_name.operand_count();
    if operands.len() <= required_operands {
        return Ok(PreparedOperands::new(operands));
    }

    let tail = &operands[required_operands..];
    let matches = spec::list_tail_spec()
        .try_get_matches_from(tail.iter().cloned())
        .map_err(|_error| list_tail_error(command_name, tail))?;

    let output_format = if matches.get_flag("json") {
        ListOutputFormat::Json
    } else if matches.get_flag("yaml") {
        ListOutputFormat::Yaml
    } else if matches.get_flag("names") {
        ListOutputFormat::Names
    } else {
        ListOutputFormat::Table
    };
    let unique = matches.get_flag("unique");

    Ok(PreparedOperands {
        operands: operands[..required_operands].to_vec(),
        mutation_options: MutationOptions::default(),
        list_options: ListOptions {
            output_format,
            unique,
        },
    })
}

pub(super) fn mutation_tail_error(command_name: CommandName, tail: &[OsString]) -> ParseControl {
    // Clap identifies that parsing failed; this replay maps the failed tail to
    // the exact diagnostic contract.
    let mut mode = MutationMode::Write;
    let mut check = false;
    for option in tail {
        match option.to_str() {
            Some("--stdout") => {
                if mode != MutationMode::Write {
                    return mutation_output_conflict(command_name);
                }
                mode = MutationMode::Stdout;
            }
            Some("--diff") => {
                if mode != MutationMode::Write {
                    return mutation_output_conflict(command_name);
                }
                mode = MutationMode::Diff;
            }
            Some("--check") => {
                if check {
                    return ParseControl::Usage(UsageError {
                        message: format!(
                            "{USAGE}envq: error: duplicate option for envq {}: --check\n",
                            command_name.as_str()
                        )
                        .into_bytes(),
                    });
                }
                check = true;
            }
            _ => {
                return ParseControl::Usage(UsageError {
                    message: command_usage_error(command_name),
                });
            }
        }
    }

    ParseControl::Usage(UsageError {
        message: command_usage_error(command_name),
    })
}

pub(super) fn list_tail_error(command_name: CommandName, tail: &[OsString]) -> ParseControl {
    // Keep diagnostics stable instead of surfacing raw clap wording.
    let mut output_format = ListOutputFormat::Table;
    let mut unique = false;
    for option in tail {
        match option.to_str() {
            Some("--json") => {
                if output_format != ListOutputFormat::Table {
                    return list_format_conflict(command_name);
                }
                output_format = ListOutputFormat::Json;
            }
            Some("--yaml") => {
                if output_format != ListOutputFormat::Table {
                    return list_format_conflict(command_name);
                }
                output_format = ListOutputFormat::Yaml;
            }
            Some("--names") => {
                if output_format != ListOutputFormat::Table {
                    return list_format_conflict(command_name);
                }
                output_format = ListOutputFormat::Names;
            }
            Some("--unique") => {
                if unique {
                    return ParseControl::Usage(UsageError {
                        message: format!(
                            "{USAGE}envq: error: duplicate option for envq {}: --unique\n",
                            command_name.as_str()
                        )
                        .into_bytes(),
                    });
                }
                unique = true;
            }
            _ => {
                return ParseControl::Usage(UsageError {
                    message: command_usage_error(command_name),
                });
            }
        }
    }

    ParseControl::Usage(UsageError {
        message: command_usage_error(command_name),
    })
}

fn mutation_output_conflict(command_name: CommandName) -> ParseControl {
    ParseControl::Usage(UsageError {
        message: format!(
            "{USAGE}envq: error: choose only one output option for envq {}\n",
            command_name.as_str()
        )
        .into_bytes(),
    })
}

fn list_format_conflict(command_name: CommandName) -> ParseControl {
    ParseControl::Usage(UsageError {
        message: format!(
            "{USAGE}envq: error: choose only one list output format for envq {}\n",
            command_name.as_str()
        )
        .into_bytes(),
    })
}
