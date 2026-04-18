//! Byte-aware formatting for `envq list`.
//!
//! The JSON and YAML writers in this module are intentionally handwritten
//! instead of using Serde. `envq` values are raw bytes, while Serde's normal
//! string model requires valid UTF-8. Using it here would either reject,
//! transform, or encode byte values that must remain byte-for-byte compatible
//! with the golden fixtures.

use std::collections::HashSet;

use super::types::{ListOptions, ListOutputFormat};

/// Formats bindings according to the list command options.
pub(super) fn list_stdout(bindings: &[(String, Vec<u8>)], options: ListOptions) -> Vec<u8> {
    let selected_bindings = if options.unique {
        unique_bindings(bindings)
    } else {
        bindings.to_vec()
    };

    match options.output_format {
        ListOutputFormat::Table => {
            let mut output = Vec::new();
            for (key, value) in selected_bindings {
                output.extend(key.as_bytes());
                output.push(b'\t');
                output.extend(value);
                output.push(b'\n');
            }
            output
        }
        ListOutputFormat::Names => {
            let mut output = Vec::new();
            for (key, _value) in selected_bindings {
                output.extend(key.as_bytes());
                output.push(b'\n');
            }
            output
        }
        ListOutputFormat::Json => json_bindings(&selected_bindings),
        ListOutputFormat::Yaml => yaml_bindings(&selected_bindings),
    }
}

fn unique_bindings(bindings: &[(String, Vec<u8>)]) -> Vec<(String, Vec<u8>)> {
    let mut seen = HashSet::new();
    let mut unique_bindings = Vec::new();
    for binding in bindings {
        let key = &binding.0;
        let value = &binding.1;
        if seen.insert(key.clone()) {
            unique_bindings.push((key.clone(), value.clone()));
        }
    }
    unique_bindings
}

fn json_bindings(bindings: &[(String, Vec<u8>)]) -> Vec<u8> {
    if bindings.is_empty() {
        return b"[]\n".to_vec();
    }

    let mut output = b"[\n".to_vec();
    for (index, binding) in bindings.iter().enumerate() {
        let key = &binding.0;
        let value = &binding.1;
        output.extend(b"  {\n    \"key\": ");
        extend_json_string(&mut output, key.as_bytes());
        output.extend(b",\n    \"value\": ");
        extend_json_string(&mut output, value);
        output.extend(b"\n  }");
        if index + 1 < bindings.len() {
            output.push(b',');
        }
        output.push(b'\n');
    }
    output.extend(b"]\n");
    output
}

fn yaml_bindings(bindings: &[(String, Vec<u8>)]) -> Vec<u8> {
    // This is the narrow YAML shape promised by the CLI contract, not a
    // general YAML serializer. It reuses JSON string quoting because the
    // quoted scalar syntax is valid YAML and keeps byte escaping identical
    // between `--json` and `--yaml`.
    if bindings.is_empty() {
        return b"[]\n".to_vec();
    }

    let mut output = Vec::new();
    for binding in bindings {
        let key = &binding.0;
        let value = &binding.1;
        output.extend(b"- key: ");
        extend_json_string(&mut output, key.as_bytes());
        output.extend(b"\n  value: ");
        extend_json_string(&mut output, value);
        output.push(b'\n');
    }
    output
}

fn extend_json_string(output: &mut Vec<u8>, value: &[u8]) {
    // This is deliberately byte-oriented: valid UTF-8 is emitted like
    // `ensure_ascii=False`, while invalid bytes are preserved as raw bytes.
    output.push(b'"');
    for byte in value {
        match *byte {
            b'"' => output.extend(b"\\\""),
            b'\\' => output.extend(b"\\\\"),
            b'\n' => output.extend(b"\\n"),
            b'\r' => output.extend(b"\\r"),
            b'\t' => output.extend(b"\\t"),
            0x08 => output.extend(b"\\b"),
            0x0c => output.extend(b"\\f"),
            0x00..=0x1f => {
                extend_control_escape(output, *byte);
            }
            other => output.push(other),
        }
    }
    output.push(b'"');
}

fn extend_control_escape(output: &mut Vec<u8>, byte: u8) {
    output.extend(b"\\u00");
    output.push(hex_digit(byte >> 4));
    output.push(hex_digit(byte & 0x0f));
}

fn hex_digit(nibble: u8) -> u8 {
    b"0123456789abcdef"[usize::from(nibble)]
}
