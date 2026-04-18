use std::fs;

use crate::context::{ReferenceContext, normalize_output};
use crate::support::{content_bytes, json_object};

#[derive(Clone, Debug)]
struct Step {
    args: Vec<String>,
    stdin: Vec<u8>,
}

#[derive(Clone, Debug)]
struct FlowScenario {
    name: &'static str,
    input: Vec<u8>,
    steps: Vec<Step>,
}

#[test]
#[ignore = "requires ENVQ_LEGACY_REFERENCE and ENVQ_LEGACY_REFERENCE_RUNNER"]
fn compare_selected_flows_with_legacy_reference() {
    let context = ReferenceContext::new();
    let scenarios = flow_scenarios();

    for scenario in scenarios {
        let tempdir = tempfile::tempdir().expect("create tempdir");
        let legacy_file = tempdir.path().join(format!("{}.legacy.env", scenario.name));
        let rs_file = tempdir.path().join(format!("{}.rs.env", scenario.name));
        fs::write(&legacy_file, &scenario.input).expect("write legacy flow input");
        fs::write(&rs_file, &scenario.input).expect("write Rust flow input");

        for (index, step) in scenario.steps.iter().enumerate() {
            let legacy_output = context.run_legacy(&step.args, &step.stdin, &legacy_file);
            let rust_output = context.run_rust(&step.args, &step.stdin, &rs_file);
            assert_eq!(
                normalize_output(legacy_output, &legacy_file),
                normalize_output(rust_output, &rs_file),
                "{} step {}",
                scenario.name,
                index + 1
            );
        }
    }
}

fn flow_scenarios() -> Vec<FlowScenario> {
    vec![
        FlowScenario {
            name: "real-life-service",
            input: content_bytes(&json_object(
                "file",
                "files/real_life_service_input.env",
                "utf-8",
            )),
            steps: vec![
                step(["list", "{path}"], b""),
                step(["set", "APP_NAME", "Checkout API v2", "{path}"], b""),
                step(
                    [
                        "set",
                        "FEATURE_FLAGS",
                        "search,payments,emails,refunds",
                        "{path}",
                    ],
                    b"",
                ),
                step(
                    ["set", "TSV_COLUMNS", "id\tname\tstatus\towner", "{path}"],
                    b"",
                ),
                step(["unset", "EMPTY", "{path}"], b""),
            ],
        },
        FlowScenario {
            name: "crlf-worker",
            input: content_bytes(&json_object(
                "file",
                "files/crlf_worker_input.escaped",
                "escaped-bytes",
            )),
            steps: vec![
                step(["set", "APP_NAME", "Worker Service", "{path}"], b""),
                step(["set", "RETRIES", "5", "{path}"], b""),
                step(
                    ["set", "TSV_PARTITIONS", "alpha\tbeta\tgamma", "{path}"],
                    b"",
                ),
                step(["list", "{path}"], b""),
            ],
        },
        FlowScenario {
            name: "complex-values",
            input: b"A=old\nB=2\n".to_vec(),
            steps: vec![
                step(
                    ["set", "A", "quote\"slash\\hash#space value", "{path}"],
                    b"",
                ),
                step(["set", "B", "-", "{path}"], b"line1\nline2\n"),
                step(["list", "{path}", "--json"], b""),
                step(
                    [
                        "set",
                        "A",
                        "quote\"slash\\hash#space value",
                        "{path}",
                        "--check",
                    ],
                    b"",
                ),
            ],
        },
    ]
}

fn step<const N: usize>(args: [&str; N], stdin: &[u8]) -> Step {
    Step {
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
        stdin: stdin.to_vec(),
    }
}
