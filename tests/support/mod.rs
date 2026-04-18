#![allow(dead_code)]

use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

pub const FIXTURE_ROOT: &str = "tests/fixtures/golden";

pub fn load_manifest(name: &str) -> Value {
    let path = Path::new(FIXTURE_ROOT).join(name);
    let text = fs::read_to_string(&path).expect("read manifest");
    serde_json::from_str(&text).expect("parse manifest")
}

pub fn cases(manifest: &Value) -> &[Value] {
    required(manifest, "cases")
        .as_array()
        .expect("cases must be array")
}

pub fn platform_applies(case: &Value) -> bool {
    match required_str(case, "platform") {
        "all" => true,
        "posix" => cfg!(unix),
        "windows" => cfg!(windows),
        other => panic!("unsupported platform: {other}"),
    }
}

pub fn prepare_fixture_file(
    case: &Value,
    input_spec: &Value,
    root: &Path,
) -> (PathBuf, Option<Vec<u8>>) {
    let env_file = root.join(format!("{}.env", case_id(case)));
    if input_spec.get("missing").and_then(Value::as_bool) == Some(true) {
        return (env_file, None);
    }
    if input_spec.get("directory").and_then(Value::as_bool) == Some(true) {
        fs::create_dir(&env_file).expect("create fixture directory input");
        return (env_file, None);
    }
    if input_spec.get("missing_parent").and_then(Value::as_bool) == Some(true) {
        return (
            root.join("missing-parent")
                .join(format!("{}.env", case_id(case))),
            None,
        );
    }

    let input_bytes = content_bytes(input_spec);
    fs::write(&env_file, &input_bytes).expect("write fixture input");
    (env_file, Some(input_bytes))
}

pub fn cli_args(case: &Value) -> Vec<String> {
    required(case, "args")
        .as_array()
        .expect("args must be array")
        .iter()
        .map(|value| value.as_str().expect("arg must be string").to_owned())
        .collect()
}

pub fn cli_args_with_path(case: &Value, env_file: &Path) -> Vec<OsString> {
    required(case, "args")
        .as_array()
        .expect("args must be array")
        .iter()
        .map(|value| {
            let argument = value.as_str().expect("arg must be string");
            OsString::from(expand_placeholders(argument, env_file))
        })
        .collect()
}

pub fn stdin_bytes(case: &Value) -> Vec<u8> {
    case.get("stdin")
        .and_then(Value::as_str)
        .map_or_else(Vec::new, |value| value.as_bytes().to_vec())
}

pub fn assert_expected_cli_file(spec: &Value, env_file: &Path, input_bytes: Option<&[u8]>) {
    if spec.get("missing").and_then(Value::as_bool) == Some(true) {
        assert!(!env_file.exists(), "expected file to be missing");
        return;
    }
    if spec.get("directory").and_then(Value::as_bool) == Some(true) {
        assert!(env_file.is_dir(), "expected path to remain a directory");
        return;
    }

    let expected = expected_bytes(spec, input_bytes);
    let actual = fs::read(env_file).expect("read cli output file");
    assert_eq!(actual, expected, "{}", env_file.display());
}

pub fn output_bytes(spec: &Value, env_file: &Path) -> Vec<u8> {
    if let Some(text) = spec.as_str() {
        return expand_placeholders(text, env_file).into_bytes();
    }
    let spec = object(spec);
    if let Some(text) = spec.get("text").and_then(Value::as_str) {
        return expand_placeholders(text, env_file).into_bytes();
    }
    content_bytes(spec)
}

pub fn expected_bytes(spec: &Value, input_bytes: Option<&[u8]>) -> Vec<u8> {
    if spec.get("same_as_input").and_then(Value::as_bool) == Some(true) {
        return input_bytes
            .expect("same_as_input requires input bytes")
            .to_vec();
    }
    content_bytes(spec)
}

pub fn content_bytes(spec: &Value) -> Vec<u8> {
    if let Some(text) = spec.get("text").and_then(Value::as_str) {
        return text.as_bytes().to_vec();
    }

    let relative_file = required_str(spec, "file");
    let format = spec
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or("utf-8");
    let path = Path::new(FIXTURE_ROOT).join(relative_file);
    match format {
        "utf-8" => fs::read(&path).expect("read utf-8 fixture"),
        "escaped-bytes" => {
            let text = fs::read_to_string(&path).expect("read escaped fixture");
            read_escaped_bytes(&text)
        }
        other => panic!("unsupported fixture format: {other}"),
    }
}

pub fn read_escaped_bytes(text: &str) -> Vec<u8> {
    let text = text
        .strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .unwrap_or(text);
    let mut output = Vec::new();
    let mut index = 0;
    while index < text.len() {
        let remaining = &text[index..];
        if remaining.starts_with("\\r") {
            output.push(b'\r');
            index += 2;
        } else if remaining.starts_with("\\n") {
            output.push(b'\n');
            index += 2;
        } else if remaining.starts_with("\\t") {
            output.push(b'\t');
            index += 2;
        } else if remaining.starts_with("\\\\") {
            output.push(b'\\');
            index += 2;
        } else if remaining.starts_with("\\x") {
            let hex = remaining
                .get(2..4)
                .unwrap_or_else(|| panic!("incomplete hex escape in escaped fixture: {text:?}"));
            let byte = u8::from_str_radix(hex, 16)
                .unwrap_or_else(|error| panic!("invalid hex escape \\x{hex}: {error}"));
            output.push(byte);
            index += 4;
        } else {
            let char = remaining
                .chars()
                .next()
                .expect("non-empty remaining escaped fixture");
            let mut buffer = [0; 4];
            output.extend(char.encode_utf8(&mut buffer).as_bytes());
            index += char.len_utf8();
        }
    }
    output
}

pub fn string_pairs(value: &Value) -> Vec<(String, Vec<u8>)> {
    value
        .as_array()
        .expect("pairs must be array")
        .iter()
        .map(|row| {
            let row = row.as_array().expect("pair must be array");
            assert_eq!(row.len(), 2, "pair must contain two items");
            (
                row[0].as_str().expect("key must be string").to_owned(),
                row[1]
                    .as_str()
                    .expect("value must be string")
                    .as_bytes()
                    .to_vec(),
            )
        })
        .collect()
}

pub fn string_list(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("list must be array")
        .iter()
        .map(|value| value.as_str().expect("item must be string").to_owned())
        .collect()
}

pub fn expand_placeholders(text: &str, path: &Path) -> String {
    text.replace("{path}", &path.display().to_string())
        .replace("{version}", env!("CARGO_PKG_VERSION"))
}

pub fn json_object(kind: &str, file: &str, format: &str) -> Value {
    let mut object = serde_json::Map::new();
    object.insert(kind.to_owned(), Value::String(file.to_owned()));
    object.insert("format".to_owned(), Value::String(format.to_owned()));
    Value::Object(object)
}

pub fn object(value: &Value) -> &Value {
    assert!(value.is_object(), "value must be object");
    value
}

pub fn required<'a>(value: &'a Value, key: &str) -> &'a Value {
    value
        .get(key)
        .unwrap_or_else(|| panic!("missing required key: {key}"))
}

pub fn required_str<'a>(value: &'a Value, key: &str) -> &'a str {
    required(value, key)
        .as_str()
        .unwrap_or_else(|| panic!("{key} must be string"))
}

pub fn case_id(case: &Value) -> &str {
    required_str(case, "id")
}

pub trait OsStrBytes {
    fn encoded_bytes(&self) -> Vec<u8>;
}

#[cfg(unix)]
impl OsStrBytes for OsStr {
    fn encoded_bytes(&self) -> Vec<u8> {
        use std::os::unix::ffi::OsStrExt;

        self.as_bytes().to_vec()
    }
}

#[cfg(not(unix))]
impl OsStrBytes for OsStr {
    fn encoded_bytes(&self) -> Vec<u8> {
        self.to_string_lossy().as_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{expand_placeholders, read_escaped_bytes};

    #[test]
    fn escaped_bytes_strip_lf_or_crlf_fixture_terminators() {
        assert_eq!(read_escaped_bytes("A\\r\\n\n"), b"A\r\n");
        assert_eq!(read_escaped_bytes("A\\r\\n\r\n"), b"A\r\n");
    }

    #[test]
    fn fixture_placeholders_expand_path_and_package_version() {
        let expanded = expand_placeholders("envq {version}: {path}", Path::new("demo.env"));
        assert_eq!(
            expanded,
            format!("envq {}: demo.env", env!("CARGO_PKG_VERSION"))
        );
    }
}
