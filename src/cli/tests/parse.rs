use super::*;

#[test]
fn global_help_version_quiet_and_help_topics() {
    let version = run([OsString::from("--version")], &mut Cursor::new(Vec::new()));
    assert_eq!(version.exit_code, ExitCode::Success);
    assert!(version.stdout.starts_with(b"envq "));

    let short_help = run([OsString::from("-h")], &mut Cursor::new(Vec::new()));
    assert_eq!(short_help.exit_code, ExitCode::Success);
    assert!(short_help.stdout.starts_with(b"usage: envq "));

    let no_args = run(Vec::<OsString>::new(), &mut Cursor::new(Vec::new()));
    assert_eq!(no_args.exit_code, ExitCode::Success);
    assert!(no_args.stdout.starts_with(b"usage: envq "));

    let quiet_only = run([OsString::from("--quiet")], &mut Cursor::new(Vec::new()));
    assert_eq!(quiet_only.exit_code, ExitCode::Success);
    assert!(quiet_only.stdout.starts_with(b"usage: envq "));

    let help = run([OsString::from("help")], &mut Cursor::new(Vec::new()));
    assert_eq!(help.exit_code, ExitCode::Success);
    assert!(help.stdout.starts_with(b"usage: envq "));

    for topic in [
        "get",
        "set",
        "clear",
        "unset",
        "remove",
        "has",
        "list",
        "completion",
        "help",
    ] {
        let result = run(
            [OsString::from("help"), OsString::from(topic)],
            &mut Cursor::new(Vec::new()),
        );
        assert_eq!(result.exit_code, ExitCode::Success, "{topic}");
        assert!(
            result
                .stdout
                .starts_with(format!("usage: envq {topic}").as_bytes()),
            "{topic}"
        );
    }

    let unknown = run(
        [OsString::from("help"), OsString::from("unknown")],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(unknown.exit_code, ExitCode::GeneralError);
    assert_eq!(unknown.stderr, b"envq: unknown help topic: unknown\n");
}

#[test]
fn command_parse_and_file_error_paths() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    let missing_file = directory.path().join("missing.env");
    let missing_parent_file = directory.path().join("missing").join(".env");
    fs::write(&env_file, b"A=1\n").expect("write env file");

    let invalid_key = run(
        [
            "get".into(),
            "1BAD".into(),
            missing_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(invalid_key.exit_code, ExitCode::ValidationError);
    assert_eq!(invalid_key.stderr, b"envq: invalid key: 1BAD\n");

    for command in ["set", "clear", "unset", "has"] {
        let mut argv = vec![OsString::from(command), OsString::from("1BAD")];
        if command == "set" {
            argv.push(OsString::from("value"));
        }
        argv.push(env_file.as_os_str().to_owned());

        let result = run(argv, &mut Cursor::new(Vec::new()));
        assert_eq!(result.exit_code, ExitCode::ValidationError, "{command}");
        assert_eq!(result.stderr, b"envq: invalid key: 1BAD\n", "{command}");
    }

    let missing = run(
        [
            "get".into(),
            "A".into(),
            missing_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(missing.exit_code, ExitCode::GeneralError);
    assert!(missing.stderr.ends_with(b": No such file or directory\n"));

    for command in ["has", "list", "unset"] {
        let mut argv = vec![OsString::from(command)];
        if command != "list" {
            argv.push(OsString::from("A"));
        }
        argv.push(missing_file.as_os_str().to_owned());

        let result = run(argv, &mut Cursor::new(Vec::new()));
        assert_eq!(result.exit_code, ExitCode::GeneralError, "{command}");
        assert!(result.stderr.ends_with(b": No such file or directory\n"));
    }

    let missing_parent = run(
        [
            "set".into(),
            "A".into(),
            "1".into(),
            missing_parent_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(missing_parent.exit_code, ExitCode::GeneralError);
    assert!(
        missing_parent
            .stderr
            .ends_with(b": No such file or directory\n")
    );

    let clear_missing_parent = run(
        [
            "clear".into(),
            "A".into(),
            missing_parent_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(clear_missing_parent.exit_code, ExitCode::GeneralError);
    assert!(
        clear_missing_parent
            .stderr
            .ends_with(b": No such file or directory\n")
    );

    let clear_missing = run(
        [
            "clear".into(),
            "A".into(),
            missing_file.as_os_str().to_owned(),
            "--stdout".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(clear_missing.exit_code, ExitCode::Success);
    assert_eq!(clear_missing.stdout, bytes_with_default_newline(b"A="));

    let directory_path = run(
        [
            "set".into(),
            "A".into(),
            "1".into(),
            directory.path().as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(directory_path.exit_code, ExitCode::GeneralError);
    assert_directory_like_error(&directory_path.stderr);

    let bad_command = run([OsString::from("wat")], &mut Cursor::new(Vec::new()));
    assert_eq!(bad_command.exit_code, ExitCode::GeneralError);
    assert!(
        bad_command
            .stderr
            .starts_with(b"usage: envq [-h] [--version] [--quiet]")
    );

    let too_many_help_operands = run(
        [
            OsString::from("help"),
            OsString::from("get"),
            OsString::from("set"),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(too_many_help_operands.exit_code, ExitCode::GeneralError);
    assert_eq!(
        too_many_help_operands.stderr,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: expected operands: envq help [COMMAND]\n"
    );

    for (command, signature) in [
        ("get", "get KEY PATH"),
        ("set", "set KEY VALUE PATH"),
        ("clear", "clear KEY PATH"),
        ("unset", "unset KEY PATH"),
        ("remove", "remove KEY PATH"),
        ("has", "has KEY PATH"),
        ("list", "list PATH"),
        ("completion", "completion {bash,zsh,fish,powershell,pwsh}"),
    ] {
        let result = run([OsString::from(command)], &mut Cursor::new(Vec::new()));
        assert_eq!(result.exit_code, ExitCode::GeneralError, "{command}");
        assert_eq!(
            result.stderr,
            format!(
                "usage: envq [-h] [--version] [--quiet] {{get,set,clear,unset,remove,has,list,completion,help}} ...\nenvq: error: expected operands: envq {signature}\n"
            )
            .as_bytes()
        );
    }

    let quiet_missing_key = run(
        [
            "--quiet".into(),
            "get".into(),
            "B".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(quiet_missing_key.exit_code, ExitCode::KeyNotFound);
    assert!(quiet_missing_key.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn non_utf8_operands_use_lossy_diagnostics() {
    use std::os::unix::ffi::OsStringExt;

    let command = OsString::from_vec(vec![0xff]);
    let result = run([command], &mut Cursor::new(Vec::new()));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert!(result.stderr.starts_with(b"usage: envq "));
    assert!(
        result
            .stderr
            .ends_with(b"invalid choice: \"\xEF\xBF\xBD\"\n")
    );

    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\n").expect("write env file");
    let invalid_key = OsString::from_vec(vec![0xff]);

    let result = run(
        ["get".into(), invalid_key, env_file.as_os_str().to_owned()],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(result.exit_code, ExitCode::ValidationError);
    assert_eq!(result.stderr, b"envq: invalid key: \xEF\xBF\xBD\n");
}

#[test]
fn internal_tail_error_replay_handles_empty_tails() {
    let mutation = super::super::tail::mutation_tail_error(CommandName::Set, &[]);
    let super::super::types::ParseControl::Usage(mutation) = mutation else {
        panic!("expected usage");
    };
    assert_eq!(
        mutation.message,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: expected operands: envq set KEY VALUE PATH\n"
    );

    let list = super::super::tail::list_tail_error(CommandName::List, &[]);
    let super::super::types::ParseControl::Usage(list) = list else {
        panic!("expected usage");
    };
    assert_eq!(
        list.message,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: expected operands: envq list PATH\n"
    );
}

#[test]
fn internal_command_operand_counts_cover_help() {
    assert_eq!(CommandName::Help.operand_count(), 0);
}

#[test]
fn mutation_tail_options_keep_contract_compatible_diagnostics() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\n").expect("write env file");

    let conflict = run(
        [
            "set".into(),
            "A".into(),
            "2".into(),
            env_file.as_os_str().to_owned(),
            "--stdout".into(),
            "--diff".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(conflict.exit_code, ExitCode::GeneralError);
    assert_eq!(
        conflict.stderr,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: choose only one output option for envq set\n"
    );

    let duplicate_check = run(
        [
            "set".into(),
            "A".into(),
            "2".into(),
            env_file.as_os_str().to_owned(),
            "--check".into(),
            "--check".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(duplicate_check.exit_code, ExitCode::GeneralError);
    assert_eq!(
        duplicate_check.stderr,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: duplicate option for envq set: --check\n"
    );

    let reverse_conflict = run(
        [
            "set".into(),
            "A".into(),
            "2".into(),
            env_file.as_os_str().to_owned(),
            "--diff".into(),
            "--stdout".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(reverse_conflict.exit_code, ExitCode::GeneralError);
    assert_eq!(
        reverse_conflict.stderr,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: choose only one output option for envq set\n"
    );

    let unknown_tail = run(
        [
            "set".into(),
            "A".into(),
            "2".into(),
            env_file.as_os_str().to_owned(),
            "--wat".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(unknown_tail.exit_code, ExitCode::GeneralError);
    assert_eq!(
        unknown_tail.stderr,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: expected operands: envq set KEY VALUE PATH\n"
    );
}
