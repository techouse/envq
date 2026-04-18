#![forbid(unsafe_code)]

use std::io::{Cursor, Write};
use std::path::Path;

use envq::cli::run;
use envq::editor::{set_value, unset_key};
use envq::parser::parse_document;
use envq::render::render_document;
use serde::Deserialize;

const MAX_DOCUMENT_LEN: usize = 4096;
const MAX_VALUE_LEN: usize = 512;
const MAX_DIFF_INPUT_LEN: usize = 4096;
const MAX_JSON_INPUT_LEN: usize = 8192;
const MAX_LIST_OUTPUT_LEN: usize = 1_048_576;
const MAX_DIFF_OUTPUT_LEN: usize = 1_048_576;

#[derive(Debug, Deserialize, Default)]
pub struct DocumentCase {
    #[serde(default)]
    document: Vec<u8>,
}

#[derive(Debug, Deserialize, Default)]
pub struct EditCase {
    #[serde(default)]
    document: Vec<u8>,
    #[serde(default)]
    value: Vec<u8>,
    #[serde(default)]
    operation: EditOperation,
}

#[derive(Clone, Copy, Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EditOperation {
    #[default]
    Set,
    Clear,
    Unset,
    SetThenUnset,
}

#[derive(Debug, Deserialize, Default)]
pub struct DiffCase {
    #[serde(default)]
    before: Vec<u8>,
    #[serde(default)]
    after: Vec<u8>,
}

pub fn run_parse_roundtrip_bytes(data: &[u8]) {
    let document_bytes = document_bytes_from_input(data);
    let document = parse_document(&document_bytes);
    let rendered = render_document(&document);

    assert_eq!(rendered, document_bytes);
}

pub fn run_edit_set_unset_bytes(data: &[u8]) {
    let case = edit_case_from_input(data);
    let document = parse_document(&case.document);
    let updated = match case.operation {
        EditOperation::Set => set_value(&document, "FUZZ_KEY", &case.value),
        EditOperation::Clear => set_value(&document, "FUZZ_KEY", b""),
        EditOperation::Unset => unset_key(&document, "FUZZ_KEY").document,
        EditOperation::SetThenUnset => {
            let set_document = set_value(&document, "FUZZ_KEY", &case.value);
            unset_key(&set_document, "FUZZ_KEY").document
        }
    };

    let rendered = render_document(&updated);
    let reparsed = parse_document(&rendered);
    assert_eq!(render_document(&reparsed), rendered);
}

pub fn run_list_output_bytes(data: &[u8]) {
    let document_bytes = document_bytes_from_input(data);
    let Ok(mut env_file) = tempfile::NamedTempFile::new() else {
        return;
    };
    if env_file.write_all(&document_bytes).is_err() || env_file.flush().is_err() {
        return;
    }

    let path = env_file.path().as_os_str().to_owned();
    for options in [
        &[][..],
        &["--json"][..],
        &["--yaml"][..],
        &["--names"][..],
        &["--unique"][..],
        &["--json", "--unique"][..],
        &["--yaml", "--unique"][..],
        &["--names", "--unique"][..],
    ] {
        let mut args = vec!["list".into(), path.clone()];
        args.extend(options.iter().map(|option| (*option).into()));
        let result = run(args, &mut Cursor::new(Vec::new()));

        assert!(result.stdout.len() <= MAX_LIST_OUTPUT_LEN);
        assert!(result.stderr.len() <= MAX_LIST_OUTPUT_LEN);
    }
}

pub fn run_diff_bytes(data: &[u8]) {
    let case = diff_case_from_input(data);
    let output = envq::fuzzing::unified_diff(Path::new("fuzz.env"), &case.before, &case.after);

    assert!(output.len() <= MAX_DIFF_OUTPUT_LEN);
}

fn document_bytes_from_input(data: &[u8]) -> Vec<u8> {
    if let Some(case) = parse_json_case::<DocumentCase>(data) {
        return truncate_vec(case.document, MAX_DOCUMENT_LEN);
    }
    truncate_slice(data, MAX_DOCUMENT_LEN).to_vec()
}

fn edit_case_from_input(data: &[u8]) -> EditCase {
    let mut case = parse_json_case::<EditCase>(data).unwrap_or_else(|| EditCase {
        document: truncate_slice(data, MAX_DOCUMENT_LEN).to_vec(),
        value: truncate_slice(data, MAX_VALUE_LEN).to_vec(),
        operation: EditOperation::Set,
    });
    case.document = truncate_vec(case.document, MAX_DOCUMENT_LEN);
    case.value = truncate_vec(case.value, MAX_VALUE_LEN);
    case
}

fn diff_case_from_input(data: &[u8]) -> DiffCase {
    let mut case = parse_json_case::<DiffCase>(data).unwrap_or_else(|| {
        let midpoint = data.len().min(MAX_DIFF_INPUT_LEN * 2) / 2;
        DiffCase {
            before: truncate_slice(&data[..midpoint], MAX_DIFF_INPUT_LEN).to_vec(),
            after: truncate_slice(&data[midpoint..], MAX_DIFF_INPUT_LEN).to_vec(),
        }
    });
    case.before = truncate_vec(case.before, MAX_DIFF_INPUT_LEN);
    case.after = truncate_vec(case.after, MAX_DIFF_INPUT_LEN);
    case
}

fn parse_json_case<'a, T>(data: &'a [u8]) -> Option<T>
where
    T: Deserialize<'a>,
{
    serde_json::from_slice(truncate_slice(data, MAX_JSON_INPUT_LEN)).ok()
}

fn truncate_vec(mut bytes: Vec<u8>, max_len: usize) -> Vec<u8> {
    bytes.truncate(max_len);
    bytes
}

fn truncate_slice(bytes: &[u8], max_len: usize) -> &[u8] {
    &bytes[..bytes.len().min(max_len)]
}
