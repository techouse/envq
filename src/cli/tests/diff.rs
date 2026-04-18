use super::*;

#[test]
fn diff_check_matches_golden_shape() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\n").expect("write env file");

    let result = run(
        [
            "set".into(),
            "A".into(),
            "2".into(),
            env_file.as_os_str().to_owned(),
            "--diff".into(),
            "--check".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );

    assert_eq!(result.exit_code, ExitCode::WouldChange);
    assert_eq!(
        result.stdout,
        format!(
            "--- {} (before)\n+++ {} (after)\n@@ -1 +1 @@\n-A=1\n+A=2\n",
            env_file.display(),
            env_file.display()
        )
        .into_bytes()
    );
}

#[test]
fn diff_output_without_check_exits_successfully() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\n").expect("write env file");

    let result = run(
        [
            "set".into(),
            "A".into(),
            "2".into(),
            env_file.as_os_str().to_owned(),
            "--diff".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(
        result.stdout,
        format!(
            "--- {} (before)\n+++ {} (after)\n@@ -1 +1 @@\n-A=1\n+A=2\n",
            env_file.display(),
            env_file.display()
        )
        .into_bytes()
    );
    assert_eq!(fs::read(&env_file).expect("read env file"), b"A=1\n");
}
