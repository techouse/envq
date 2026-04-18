use std::ffi::OsString;
use std::fs;
use std::io::{self, Cursor, Read};

use envq::cli::run;
use envq::model::{ExitCode, Line};
use envq::parser::parse_document;

struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "stdin failed"))
    }
}

fn assert_directory_like_error(stderr: &[u8]) {
    let message = String::from_utf8_lossy(stderr);
    assert!(
        stderr.ends_with(b": Is a directory\n")
            || stderr.ends_with(b": Permission denied\n")
            || stderr.ends_with(b": File exists\n")
            || message.contains("Access is denied")
            || message.contains("The parameter is incorrect"),
        "{}",
        message
    );
}

#[test]
fn cli_edge_paths_exercised_in_golden_target() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\nB=two\n").expect("write env file");

    let has_existing = run(
        ["has".into(), "A".into(), env_file.as_os_str().to_owned()],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(has_existing.exit_code, ExitCode::Success);

    let has_missing = run(
        [
            "has".into(),
            "MISSING".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(has_missing.exit_code, ExitCode::KeyNotFound);

    let unset_missing = run(
        [
            "unset".into(),
            "MISSING".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(unset_missing.exit_code, ExitCode::KeyNotFound);

    let diff = run(
        [
            "set".into(),
            "A".into(),
            "2".into(),
            env_file.as_os_str().to_owned(),
            "--diff".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(diff.exit_code, ExitCode::Success);
    assert!(diff.stdout.starts_with(b"--- "));

    let stdin_error = run(
        [
            "set".into(),
            "A".into(),
            "-".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut FailingReader,
    );
    assert_eq!(stdin_error.exit_code, ExitCode::GeneralError);
    assert_eq!(stdin_error.stderr, b"envq: stdin: stdin failed\n");

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

    let no_change = run(
        [
            "set".into(),
            "A".into(),
            "1".into(),
            env_file.as_os_str().to_owned(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(no_change.exit_code, ExitCode::Success);
}

#[test]
fn help_usage_list_and_parser_edges_exercised_in_golden_target() {
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
        assert!(result.stdout.starts_with(b"usage: envq "));
    }

    for command in [
        "get",
        "set",
        "clear",
        "unset",
        "remove",
        "has",
        "list",
        "completion",
    ] {
        let result = run([OsString::from(command)], &mut Cursor::new(Vec::new()));
        assert_eq!(result.exit_code, ExitCode::GeneralError, "{command}");
        assert!(result.stderr.starts_with(b"usage: envq "));
    }

    let list_file = tempfile::NamedTempFile::new().expect("create temp file");
    fs::write(
        list_file.path(),
        b"RAW=\"quote\\\"\\\\\\n\\r\\t\x08\x0c\x01\"\n",
    )
    .expect("write list env file");

    let json = run(
        [
            "list".into(),
            list_file.path().as_os_str().to_owned(),
            "--json".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(json.exit_code, ExitCode::Success);
    assert!(json.stdout.contains(&b'\\'));

    let empty_list_file = tempfile::NamedTempFile::new().expect("create empty env file");
    let empty_json = run(
        [
            "list".into(),
            empty_list_file.path().as_os_str().to_owned(),
            "--json".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(empty_json.exit_code, ExitCode::Success);
    assert_eq!(empty_json.stdout, b"[]\n");

    let document = parse_document(
        br##"A="value"#comment
B="tail\
"##,
    );
    let Line::Binding(first) = &document.lines[0] else {
        panic!("expected binding");
    };
    assert_eq!(first.inline_comment.as_deref(), Some(&b"#comment"[..]));
    assert!(matches!(document.lines[1], Line::Invalid(_)));
}
