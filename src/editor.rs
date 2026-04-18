//! Pure document editing operations.
//!
//! These functions do not perform file I/O. They transform parsed documents and
//! leave command-specific output decisions to the CLI layer.

use crate::model::{BindingLine, Document, Line, QuoteType};
use crate::render::{EncodedValue, encode_value_auto, encode_value_double};

/// Result of removing a key from a document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsetResult {
    /// Updated document, or the original document when nothing was removed.
    pub document: Document,
    /// Number of bindings removed.
    pub removed_count: usize,
}

/// Returns the first matching binding value.
#[must_use]
pub fn get_value<'a>(document: &'a Document, key: &str) -> Option<&'a [u8]> {
    document.lines.iter().find_map(|line| match line {
        Line::Binding(binding) if binding.key == key => Some(binding.value.as_slice()),
        _ => None,
    })
}

/// Returns whether any binding with `key` exists.
#[must_use]
pub fn has_key(document: &Document, key: &str) -> bool {
    get_value(document, key).is_some()
}

/// Lists binding keys and resolved values in file order.
#[must_use]
pub fn list_bindings(document: &Document) -> Vec<(String, Vec<u8>)> {
    document
        .lines
        .iter()
        .filter_map(|line| match line {
            Line::Binding(binding) => Some((binding.key.clone(), binding.value.clone())),
            _ => None,
        })
        .collect()
}

/// Sets the first matching binding or appends a new binding.
#[must_use]
pub fn set_value(document: &Document, key: &str, value: &[u8]) -> Document {
    let mut new_lines = Vec::with_capacity(document.lines.len() + 1);
    let mut updated = false;

    for line in &document.lines {
        if let Line::Binding(binding) = line
            && !updated
            && binding.key == key
        {
            let encoded_value = encode_value_for_existing_line(value, binding);
            new_lines.push(Line::Binding(BindingLine {
                export: binding.export,
                key: binding.key.clone(),
                value: value.to_vec(),
                raw_value: encoded_value.text,
                quote_type: encoded_value.quote_type,
                prefix: binding.prefix.clone(),
                suffix: binding.suffix.clone(),
                inline_comment: binding.inline_comment.clone(),
                original_text: binding.original_text.clone(),
                newline: binding.newline.clone(),
            }));
            updated = true;
        } else {
            new_lines.push(line.clone());
        }
    }

    if updated {
        return Document {
            lines: ensure_terminal_newline(new_lines, &document.preferred_newline),
            preferred_newline: document.preferred_newline.clone(),
        };
    }

    let mut lines_for_append = ensure_terminal_newline(new_lines, &document.preferred_newline);
    let encoded_value = encode_value_auto(value);
    let mut prefix = key.as_bytes().to_vec();
    prefix.push(b'=');
    let mut original_text = prefix.clone();
    original_text.extend(&encoded_value.text);
    lines_for_append.push(Line::Binding(BindingLine {
        export: false,
        key: key.to_owned(),
        value: value.to_vec(),
        raw_value: encoded_value.text,
        quote_type: encoded_value.quote_type,
        prefix,
        suffix: Vec::new(),
        inline_comment: None,
        original_text,
        newline: document.preferred_newline.clone(),
    }));

    Document {
        lines: lines_for_append,
        preferred_newline: document.preferred_newline.clone(),
    }
}

/// Removes all bindings with `key`.
#[must_use]
pub fn unset_key(document: &Document, key: &str) -> UnsetResult {
    let mut kept_lines = Vec::with_capacity(document.lines.len());
    let mut removed_count = 0;

    for line in &document.lines {
        if matches!(line, Line::Binding(binding) if binding.key == key) {
            removed_count += 1;
        } else {
            kept_lines.push(line.clone());
        }
    }

    if removed_count == 0 {
        return UnsetResult {
            document: document.clone(),
            removed_count,
        };
    }

    if kept_lines.is_empty() {
        return UnsetResult {
            document: Document {
                lines: Vec::new(),
                preferred_newline: document.preferred_newline.clone(),
            },
            removed_count,
        };
    }

    UnsetResult {
        document: Document {
            lines: ensure_terminal_newline(kept_lines, &document.preferred_newline),
            preferred_newline: document.preferred_newline.clone(),
        },
        removed_count,
    }
}

fn ensure_terminal_newline(mut lines: Vec<Line>, preferred_newline: &[u8]) -> Vec<Line> {
    let Some(last_line) = lines.last() else {
        return lines;
    };
    if !last_line.newline().is_empty() {
        return lines;
    }

    let adjusted_last = last_line.with_newline(preferred_newline.to_vec());
    let last_index = lines.len() - 1;
    lines[last_index] = adjusted_last;
    lines
}

fn encode_value_for_existing_line(value: &[u8], line: &BindingLine) -> EncodedValue {
    let encoded_value = encode_value_auto(value);
    // Preserve suffix/comment meaning. An otherwise safe unquoted value can
    // absorb preserved suffix bytes, so quote it when that would be ambiguous.
    if encoded_value.quote_type == QuoteType::None
        && unquoted_value_would_absorb_suffix(value, &line.suffix)
    {
        return encode_value_double(value);
    }
    encoded_value
}

fn unquoted_value_would_absorb_suffix(value: &[u8], suffix: &[u8]) -> bool {
    if suffix.is_empty() {
        return false;
    }
    if suffix.iter().all(|byte| matches!(byte, b' ' | b'\t')) {
        return true;
    }
    value.is_empty() && lstrip_horizontal(suffix).starts_with(b"#")
}

fn lstrip_horizontal(text: &[u8]) -> &[u8] {
    let start = text
        .iter()
        .position(|byte| !matches!(byte, b' ' | b'\t'))
        .unwrap_or(text.len());
    &text[start..]
}

#[cfg(test)]
mod tests;
