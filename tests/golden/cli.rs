use std::io::Cursor;

use envq::cli::run;

use crate::support::{
    assert_expected_cli_file, case_id, cases, cli_args_with_path, load_manifest, object,
    output_bytes, platform_applies, prepare_fixture_file, required, stdin_bytes,
};

#[test]
fn cli_golden_fixtures() {
    let manifest = load_manifest("cli.json");
    for case in cases(&manifest) {
        if !platform_applies(case) {
            continue;
        }

        let tempdir = tempfile::tempdir().expect("create tempdir");
        let input_spec = object(required(case, "input"));
        let (env_file, input_bytes) = prepare_fixture_file(case, input_spec, tempdir.path());
        let args = cli_args_with_path(case, &env_file);
        let result = run(args, &mut Cursor::new(stdin_bytes(case)));
        let expect = object(required(case, "expect"));

        let expected_exit = required(expect, "exit_code")
            .as_i64()
            .expect("exit_code must be integer") as i32;
        assert_eq!(result.exit_code.code(), expected_exit, "{}", case_id(case));
        assert_eq!(
            result.stdout,
            output_bytes(required(expect, "stdout"), &env_file),
            "{} stdout",
            case_id(case)
        );
        assert_eq!(
            result.stderr,
            output_bytes(required(expect, "stderr"), &env_file),
            "{} stderr",
            case_id(case)
        );

        assert_expected_cli_file(
            object(required(expect, "file")),
            &env_file,
            input_bytes.as_deref(),
        );
    }
}
