use super::*;

#[test]
fn normalized_os_error_messages_cover_common_error_kinds() {
    use std::path::Path;

    assert_eq!(
        super::super::util::os_error_message(
            Path::new("x.env"),
            &io::Error::from(io::ErrorKind::PermissionDenied),
        ),
        b"envq: x.env: Permission denied\n"
    );
    assert_eq!(
        super::super::util::os_error_message(
            Path::new("x.env"),
            &io::Error::from(io::ErrorKind::AlreadyExists),
        ),
        b"envq: x.env: File exists\n"
    );
}

#[test]
fn mutation_write_errors_are_reported() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let target_directory = directory.path().join("target.env");
    fs::create_dir(&target_directory).expect("create target directory");
    let document = Document {
        lines: crate::parser::parse_document(b"A=2\n").lines,
        preferred_newline: b"\n".to_vec(),
    };

    let result = super::super::execute::finish_mutation(
        &target_directory,
        b"A=1\n",
        &document,
        MutationOptions {
            mode: MutationMode::Write,
            check: false,
        },
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_directory_like_error(&result.stderr);
}
