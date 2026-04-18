use super::*;

#[test]
fn list_empty_and_escape_outputs() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let empty_file = directory.path().join("empty.env");
    fs::write(&empty_file, b"").expect("write empty env file");

    let empty_json = run(
        [
            "list".into(),
            empty_file.as_os_str().to_owned(),
            "--json".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(empty_json.stdout, b"[]\n");

    let empty_yaml = run(
        [
            "list".into(),
            empty_file.as_os_str().to_owned(),
            "--yaml".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(empty_yaml.stdout, b"[]\n");

    let empty_unique_table = super::super::listing::list_stdout(
        &[],
        ListOptions {
            output_format: ListOutputFormat::Table,
            unique: true,
        },
    );
    assert!(empty_unique_table.is_empty());

    let escaped_json = super::super::listing::list_stdout(
        &[("RAW".to_owned(), b"quote\"\\\n\r\t\x08\x0c\x01".to_vec())],
        ListOptions {
            output_format: ListOutputFormat::Json,
            unique: false,
        },
    );
    assert_eq!(
        escaped_json,
        b"[\n  {\n    \"key\": \"RAW\",\n    \"value\": \"quote\\\"\\\\\\n\\r\\t\\b\\f\\u0001\"\n  }\n]\n"
    );

    let json_two_items = super::super::listing::list_stdout(
        &[
            ("A".to_owned(), b"1".to_vec()),
            ("B".to_owned(), b"2".to_vec()),
        ],
        ListOptions {
            output_format: ListOutputFormat::Json,
            unique: false,
        },
    );
    assert_eq!(
        json_two_items,
        b"[\n  {\n    \"key\": \"A\",\n    \"value\": \"1\"\n  },\n  {\n    \"key\": \"B\",\n    \"value\": \"2\"\n  }\n]\n"
    );

    let yaml_two_items = super::super::listing::list_stdout(
        &[
            ("A".to_owned(), b"1".to_vec()),
            ("B".to_owned(), b"2".to_vec()),
        ],
        ListOptions {
            output_format: ListOutputFormat::Yaml,
            unique: false,
        },
    );
    assert_eq!(
        yaml_two_items,
        b"- key: \"A\"\n  value: \"1\"\n- key: \"B\"\n  value: \"2\"\n"
    );
}

#[test]
fn list_json_yaml_and_invalid_bytes() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"RAW=\xff\n").expect("write env file");

    let json = run(
        [
            "list".into(),
            env_file.as_os_str().to_owned(),
            "--json".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(
        json.stdout,
        b"[\n  {\n    \"key\": \"RAW\",\n    \"value\": \"\xff\"\n  }\n]\n"
    );

    let yaml = run(
        [
            "list".into(),
            env_file.as_os_str().to_owned(),
            "--yaml".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(yaml.stdout, b"- key: \"RAW\"\n  value: \"\xff\"\n");
}

#[test]
fn list_tail_options_keep_contract_compatible_diagnostics() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let env_file = directory.path().join(".env");
    fs::write(&env_file, b"A=1\nB=2\nA=3\n").expect("write env file");

    let names_unique = run(
        [
            "list".into(),
            env_file.as_os_str().to_owned(),
            "--names".into(),
            "--unique".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(names_unique.exit_code, ExitCode::Success);
    assert_eq!(names_unique.stdout, b"A\nB\n");

    let unique_table = run(
        [
            "list".into(),
            env_file.as_os_str().to_owned(),
            "--unique".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(unique_table.exit_code, ExitCode::Success);
    assert_eq!(unique_table.stdout, b"A\t1\nB\t2\n");

    let format_conflict = run(
        [
            "list".into(),
            env_file.as_os_str().to_owned(),
            "--json".into(),
            "--yaml".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(format_conflict.exit_code, ExitCode::GeneralError);
    assert_eq!(
        format_conflict.stderr,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: choose only one list output format for envq list\n"
    );

    let duplicate_unique = run(
        [
            "list".into(),
            env_file.as_os_str().to_owned(),
            "--unique".into(),
            "--unique".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(duplicate_unique.exit_code, ExitCode::GeneralError);
    assert_eq!(
        duplicate_unique.stderr,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: duplicate option for envq list: --unique\n"
    );

    for first in ["--yaml", "--names"] {
        let conflict = run(
            [
                "list".into(),
                env_file.as_os_str().to_owned(),
                first.into(),
                "--json".into(),
            ],
            &mut Cursor::new(Vec::new()),
        );
        assert_eq!(conflict.exit_code, ExitCode::GeneralError);
        assert_eq!(
            conflict.stderr,
            b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: choose only one list output format for envq list\n"
        );
    }

    let names_conflict = run(
        [
            "list".into(),
            env_file.as_os_str().to_owned(),
            "--json".into(),
            "--names".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(names_conflict.exit_code, ExitCode::GeneralError);
    assert_eq!(
        names_conflict.stderr,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: choose only one list output format for envq list\n"
    );

    let unknown_tail = run(
        [
            "list".into(),
            env_file.as_os_str().to_owned(),
            "--wat".into(),
        ],
        &mut Cursor::new(Vec::new()),
    );
    assert_eq!(unknown_tail.exit_code, ExitCode::GeneralError);
    assert_eq!(
        unknown_tail.stderr,
        b"usage: envq [-h] [--version] [--quiet] {get,set,clear,unset,remove,has,list,completion,help} ...\nenvq: error: expected operands: envq list PATH\n"
    );
}
