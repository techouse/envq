use super::*;

#[test]
fn clap_spec_matches_documented_command_surface() {
    let command = spec::command_spec();
    let command_names = command
        .get_subcommands()
        .map(clap::Command::get_name)
        .collect::<Vec<_>>();
    assert_eq!(command_names.as_slice(), COMMAND_NAMES);

    for command_name in ["set", "clear", "unset", "remove"] {
        let command = command
            .get_subcommands()
            .find(|command| command.get_name() == command_name)
            .expect("mutation command in clap spec");
        assert_contains_long_options(command, MUTATION_OUTPUT_OPTIONS);
        assert_contains_long_options(command, &["check"]);
    }

    let list = command
        .get_subcommands()
        .find(|command| command.get_name() == "list")
        .expect("list command in clap spec");
    assert_contains_long_options(list, LIST_FORMAT_OPTIONS);
    assert_contains_long_options(list, &["unique"]);
}

#[test]
fn completion_shell_validation_uses_clap_complete_output() {
    for shell in ["bash", "zsh", "fish", "powershell", "pwsh"] {
        let result = run(
            [OsString::from("completion"), OsString::from(shell)],
            &mut Cursor::new(Vec::new()),
        );
        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.stderr.is_empty());
        assert_completion_contract(shell, &result.stdout);
    }

    let result = run(
        [OsString::from("completion"), OsString::from("cmd")],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(
        result.stderr,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: unsupported completion shell: cmd\n"
    );
}

#[test]
fn completion_shell_name_mapping_rejects_unknown_values() {
    assert_eq!(
        super::super::completion::completion_shell_from_name("bash"),
        Some(CompletionShell::Bash)
    );
    assert_eq!(
        super::super::completion::completion_shell_from_name("zsh"),
        Some(CompletionShell::Zsh)
    );
    assert_eq!(
        super::super::completion::completion_shell_from_name("fish"),
        Some(CompletionShell::Fish)
    );
    assert_eq!(
        super::super::completion::completion_shell_from_name("powershell"),
        Some(CompletionShell::PowerShell)
    );
    assert_eq!(
        super::super::completion::completion_shell_from_name("pwsh"),
        Some(CompletionShell::PowerShell)
    );
    assert_eq!(
        super::super::completion::completion_shell_from_name("cmd"),
        None
    );
}

#[test]
fn powershell_case_value_insertion_reports_success() {
    let mut script = "'envq;completion' {\n            break\n        }\n".to_owned();

    let inserted = super::super::completion::insert_powershell_case_values(
        &mut script,
        "'envq;completion' {",
        &["bash", "pwsh"],
    );

    assert!(inserted);
    assert!(script.contains("[CompletionResult]::new('bash'"));
    assert!(script.contains("[CompletionResult]::new('pwsh'"));
}

#[test]
fn powershell_case_value_insertion_reports_missing_layout() {
    let mut missing_case = "'envq' {\n            break\n        }\n".to_owned();
    assert!(!super::super::completion::insert_powershell_case_values(
        &mut missing_case,
        "'envq;completion' {",
        &["bash"],
    ));

    let mut missing_break = "'envq;completion' {\n        }\n".to_owned();
    assert!(!super::super::completion::insert_powershell_case_values(
        &mut missing_break,
        "'envq;completion' {",
        &["bash"],
    ));
}

#[test]
fn generated_powershell_completion_contains_patched_positionals() {
    let script = super::super::completion::completion_script(CompletionShell::PowerShell);
    let script = String::from_utf8(script).expect("PowerShell script is UTF-8");

    assert_powershell_case_contains(&script, "'envq;completion' {", "powershell");
    assert_powershell_case_contains(&script, "'envq;completion' {", "pwsh");
    assert_powershell_case_contains(&script, "'envq;help' {", "completion");
}

#[test]
fn powershell_positional_patch_preserves_unexpected_non_utf8_output() {
    let mut output = vec![b'p', 0xff, b's'];

    super::super::completion::append_powershell_positional_values(&mut output);

    assert_eq!(output, vec![b'p', 0xff, b's']);
}

fn assert_completion_contract(shell: &str, stdout: &[u8]) {
    let output = String::from_utf8_lossy(stdout);
    for expected in [
        "get",
        "set",
        "clear",
        "unset",
        "remove",
        "has",
        "list",
        "completion",
        "help",
        "bash",
        "zsh",
        "fish",
        "powershell",
        "pwsh",
    ] {
        assert!(output.contains(expected), "{shell} missing {expected}");
    }
    let option_prefix = if shell == "fish" { "-l " } else { "--" };
    for option in [
        "help", "version", "quiet", "stdout", "diff", "check", "json", "yaml", "names", "unique",
    ] {
        let expected = format!("{option_prefix}{option}");
        assert!(output.contains(&expected), "{shell} missing {expected}");
    }
}

fn assert_powershell_case_contains(script: &str, case_header: &str, expected: &str) {
    let case_start = script.find(case_header).expect("PowerShell case header");
    let case_body_start = case_start + case_header.len();
    let relative_break = script[case_body_start..]
        .find("\n            break")
        .expect("PowerShell case break");
    let case_body = &script[case_body_start..case_body_start + relative_break];
    assert!(
        case_body.contains(&format!("[CompletionResult]::new('{expected}'")),
        "{case_header} missing {expected}"
    );
}

fn assert_contains_long_options(command: &clap::Command, expected: &[&str]) {
    let long_options = command
        .get_arguments()
        .filter_map(clap::Arg::get_long)
        .collect::<Vec<_>>();
    for option in expected {
        assert!(
            long_options.contains(option),
            "{} missing --{option}",
            command.get_name()
        );
    }
}
