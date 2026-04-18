use super::*;

#[test]
fn get_and_missing_key() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\n").expect("write env file");

    let result = run(
        ["get".into(), "A".into(), env_file.as_os_str().to_owned()],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(
        result,
        CliResult {
            exit_code: ExitCode::Success,
            stdout: b"1".to_vec(),
            stderr: Vec::new(),
        }
    );

    let result = run(
        ["get".into(), "B".into(), env_file.as_os_str().to_owned()],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(result.exit_code, ExitCode::KeyNotFound);
    assert_eq!(result.stderr, b"envq: B: key not found\n");
}

#[test]
fn has_clear_unset_remove_and_noop_write_paths() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\nB=2\n").expect("write env file");

    let has = run(
        ["has".into(), "A".into(), env_file.as_os_str().to_owned()],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(has, CliResult::success());

    let has_missing = run(
        [
            "has".into(),
            "MISSING".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(has_missing.exit_code, ExitCode::KeyNotFound);
    assert!(has_missing.stderr.is_empty());

    let noop_set = run(
        [
            "set".into(),
            "A".into(),
            "1".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(noop_set, CliResult::success());
    assert_eq!(fs::read(&env_file).expect("read env file"), b"A=1\nB=2\n");

    let clear = run(
        [
            "clear".into(),
            "B".into(),
            env_file.as_os_str().to_owned(),
            "--stdout".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(clear.exit_code, ExitCode::Success);
    assert_eq!(clear.stdout, b"A=1\nB=\n");

    let unset = run(
        [
            "unset".into(),
            "B".into(),
            env_file.as_os_str().to_owned(),
            "--stdout".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(unset.exit_code, ExitCode::Success);
    assert_eq!(unset.stdout, b"A=1\n");

    let remove = run(
        [
            "remove".into(),
            "A".into(),
            env_file.as_os_str().to_owned(),
            "--stdout".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(remove.exit_code, ExitCode::Success);
    assert_eq!(remove.stdout, b"B=2\n");

    let unset_missing = run(
        [
            "unset".into(),
            "MISSING".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(unset_missing.exit_code, ExitCode::KeyNotFound);
    assert_eq!(unset_missing.stderr, b"envq: MISSING: key not found\n");
}

#[test]
fn set_stdout_does_not_write() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\n").expect("write env file");

    let result = run(
        [
            "set".into(),
            "A".into(),
            "2".into(),
            env_file.as_os_str().to_owned(),
            "--stdout".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, b"A=2\n");
    assert_eq!(fs::read(&env_file).expect("read env file"), b"A=1\n");
}

#[test]
fn set_can_create_missing_file_when_parent_exists() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");

    let result = run(
        [
            "set".into(),
            "A".into(),
            "1".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );

    assert_eq!(result, CliResult::success());
    assert_eq!(
        fs::read(&env_file).expect("read env file"),
        bytes_with_default_newline(b"A=1")
    );
}

#[test]
fn set_reads_stdin_exactly() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"API_KEY=old\n").expect("write env file");

    let result = run(
        [
            "set".into(),
            "API_KEY".into(),
            "-".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(b"line1\nline2\n".to_vec()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(
        fs::read(&env_file).expect("read env file"),
        br#"API_KEY="line1\nline2\n"
"#
    );
}

#[test]
fn set_reports_stdin_read_errors() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\n").expect("write env file");

    let result = run(
        [
            "set".into(),
            "A".into(),
            "-".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut FailingReader,
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(result.stderr, b"envq: stdin: stdin failed\n");
}

#[test]
fn value_named_stdout_is_an_operand_before_path() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\n").expect("write env file");

    let result = run(
        [
            "set".into(),
            "A".into(),
            "--stdout".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(fs::read(&env_file).expect("read env file"), b"A=--stdout\n");
}

#[test]
fn check_modes_cover_changed_and_unchanged_outputs() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\n").expect("write env file");

    let unchanged = run(
        [
            "set".into(),
            "A".into(),
            "1".into(),
            env_file.as_os_str().to_owned(),
            "--check".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(unchanged, CliResult::success());

    let changed_stdout = run(
        [
            "set".into(),
            "A".into(),
            "2".into(),
            env_file.as_os_str().to_owned(),
            "--stdout".into(),
            "--check".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(changed_stdout.exit_code, ExitCode::WouldChange);
    assert_eq!(changed_stdout.stdout, b"A=2\n");
    assert_eq!(fs::read(&env_file).expect("read env file"), b"A=1\n");
}
