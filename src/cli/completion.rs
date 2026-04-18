use std::ffi::OsStr;

use clap_complete::aot::{Shell as ClapShell, generate};

use super::help::USAGE;
use super::spec;
use super::types::{COMMAND_NAMES, CompletionShell, ParseControl, UsageError};
use super::util::os_string_lossy;

/// Validates the requested completion shell while preserving custom diagnostics.
pub(super) fn completion_shell(value: &OsStr) -> Result<CompletionShell, ParseControl> {
    let matches = spec::completion_shell_spec()
        .try_get_matches_from([value.to_os_string()])
        .map_err(|_error| {
            ParseControl::Usage(UsageError {
                message: format!(
                    "{USAGE}envq: error: unsupported completion shell: {}\n",
                    os_string_lossy(value)
                )
                .into_bytes(),
            })
        })?;

    let shell = matches
        .get_one::<String>("shell")
        .expect("required shell value validated by clap")
        .as_str();
    match shell {
        "bash" => Ok(CompletionShell::Bash),
        "zsh" => Ok(CompletionShell::Zsh),
        "fish" => Ok(CompletionShell::Fish),
        "powershell" | "pwsh" => Ok(CompletionShell::PowerShell),
        _ => unreachable!("shell value validated by clap"),
    }
}

/// Generates the completion script for a supported shell.
pub(super) fn completion_script(shell: CompletionShell) -> Vec<u8> {
    let mut command = spec::command_spec();
    let mut output = Vec::new();
    generate(clap_shell(shell), &mut command, "envq", &mut output);
    if shell == CompletionShell::Fish {
        append_fish_positional_values(&mut output);
    } else if shell == CompletionShell::PowerShell {
        append_powershell_positional_values(&mut output);
    }
    output
}

fn clap_shell(shell: CompletionShell) -> ClapShell {
    match shell {
        CompletionShell::Bash => ClapShell::Bash,
        CompletionShell::Zsh => ClapShell::Zsh,
        CompletionShell::Fish => ClapShell::Fish,
        CompletionShell::PowerShell => ClapShell::PowerShell,
    }
}

fn append_fish_positional_values(output: &mut Vec<u8>) {
    // `clap_complete` 4.6 documents that its fish generator only handles named
    // options. Keep the generated script as the base, then add envq's finite
    // positional choices from the same spec constants.
    output.extend(
        format!(
            "complete -c envq -n \"__fish_envq_using_subcommand completion\" -f -a \"{}\"\n",
            spec::COMPLETION_SHELLS.join(" ")
        )
        .as_bytes(),
    );
    output.extend(
        format!(
            "complete -c envq -n \"__fish_envq_using_subcommand help\" -f -a \"{}\"\n",
            COMMAND_NAMES.join(" ")
        )
        .as_bytes(),
    );
}

pub(super) fn append_powershell_positional_values(output: &mut Vec<u8>) {
    // `clap_complete` 4.6's PowerShell generator does not emit finite
    // positional value choices. Patch those generated switch cases in place so
    // `envq completion` and `envq help` complete their documented values.
    let Ok(script) = std::str::from_utf8(output) else {
        return;
    };
    let mut script = script.to_owned();
    let _completion_case_found =
        insert_powershell_case_values(&mut script, "'envq;completion' {", spec::COMPLETION_SHELLS);
    let _help_case_found =
        insert_powershell_case_values(&mut script, "'envq;help' {", COMMAND_NAMES);
    *output = script.into_bytes();
}

pub(super) fn insert_powershell_case_values(
    script: &mut String,
    case_header: &str,
    values: &[&str],
) -> bool {
    let Some(case_start) = script.find(case_header) else {
        return false;
    };
    let case_body_start = case_start + case_header.len();
    let Some(relative_break) = script[case_body_start..].find("\n            break") else {
        return false;
    };
    let insert_at = case_body_start + relative_break;
    script.insert_str(insert_at, &powershell_completion_results(values));
    true
}

fn powershell_completion_results(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| {
            let value = value.replace('\'', "''");
            format!(
                "\n            [CompletionResult]::new('{value}', '{value}', [CompletionResultType]::ParameterValue, '{value}')"
            )
        })
        .collect()
}
