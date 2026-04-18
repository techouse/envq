use std::fs;

use crate::context::{ReferenceContext, normalize_output};
use crate::support::{
    case_id, cases, cli_args, load_manifest, object, platform_applies, prepare_fixture_file,
    required, stdin_bytes,
};

#[test]
#[ignore = "requires ENVQ_LEGACY_REFERENCE and ENVQ_LEGACY_REFERENCE_RUNNER"]
fn compare_cli_golden_cases_with_legacy_reference() {
    let context = ReferenceContext::new();
    let manifest = load_manifest("cli.json");

    for case in cases(&manifest) {
        if !platform_applies(case) {
            continue;
        }
        if matches!(
            case_id(case),
            "cli.version"
                | "cli.usage-completion-missing-shell"
                | "cli.help-empty-args"
                | "cli.help-flag"
                | "cli.completion-bash"
                | "cli.completion-zsh"
                | "cli.completion-fish"
                | "cli.completion-powershell"
                | "cli.completion-pwsh"
        ) {
            continue;
        }

        let tempdir = tempfile::tempdir().expect("create tempdir");
        let legacy_root = tempdir.path().join("legacy");
        let rs_root = tempdir.path().join("rs");
        fs::create_dir(&legacy_root).expect("create legacy temp root");
        fs::create_dir(&rs_root).expect("create Rust temp root");

        let input_spec = object(required(case, "input"));
        let legacy_file = prepare_fixture_file(case, input_spec, &legacy_root).0;
        let rs_file = prepare_fixture_file(case, input_spec, &rs_root).0;
        let args = cli_args(case);
        let stdin = stdin_bytes(case);

        let legacy_output = context.run_legacy(&args, &stdin, &legacy_file);
        let rust_output = context.run_rust(&args, &stdin, &rs_file);

        assert_eq!(
            normalize_output(legacy_output, &legacy_file),
            normalize_output(rust_output, &rs_file),
            "{}",
            case_id(case)
        );
    }
}
