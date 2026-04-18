//! Byte-preserving rendering and value encoding.

use crate::model::{Document, Line, QuoteType};

/// Encoded binding value plus the quote form chosen for rendering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedValue {
    /// Source bytes for the value token, including quotes when present.
    pub text: Vec<u8>,
    /// Quote form represented by `text`.
    pub quote_type: QuoteType,
}

/// Encodes safely unquoted values as-is and double-quotes everything else.
#[must_use]
pub fn encode_value_auto(value: &[u8]) -> EncodedValue {
    if is_safe_unquoted_value(value) {
        return EncodedValue {
            text: value.to_vec(),
            quote_type: QuoteType::None,
        };
    }
    encode_value_double(value)
}

/// Encodes a value as a double-quoted token using envq's supported escapes.
#[must_use]
pub fn encode_value_double(value: &[u8]) -> EncodedValue {
    let mut text = Vec::with_capacity(value.len() + 2);
    text.push(b'"');
    text.extend(escape_double_quoted(value));
    text.push(b'"');
    EncodedValue {
        text,
        quote_type: QuoteType::Double,
    }
}

/// Renders a document back to bytes.
#[must_use]
pub fn render_document(document: &Document) -> Vec<u8> {
    let mut rendered = Vec::new();
    for line in &document.lines {
        render_line(line, &mut rendered);
    }
    rendered
}

fn render_line(line: &Line, rendered: &mut Vec<u8>) {
    match line {
        Line::Binding(line) => {
            // Bindings render from stored raw pieces so unrelated syntax survives.
            rendered.extend(&line.prefix);
            rendered.extend(&line.raw_value);
            rendered.extend(&line.suffix);
            rendered.extend(&line.newline);
        }
        Line::Blank(line) => {
            rendered.extend(&line.text);
            rendered.extend(&line.newline);
        }
        Line::Comment(line) => {
            rendered.extend(&line.text);
            rendered.extend(&line.newline);
        }
        Line::Invalid(line) => {
            rendered.extend(&line.text);
            rendered.extend(&line.newline);
        }
    }
}

fn is_safe_unquoted_value(value: &[u8]) -> bool {
    value.iter().all(|byte| {
        !matches!(
            byte,
            b' ' | b'\t' | b'\r' | b'\n' | b'#' | b'\'' | b'"' | b'\\'
        ) && *byte >= 32
    })
}

fn escape_double_quoted(value: &[u8]) -> Vec<u8> {
    let mut escaped = Vec::with_capacity(value.len());
    for byte in value {
        match *byte {
            b'\\' => escaped.extend(b"\\\\"),
            b'"' => escaped.extend(b"\\\""),
            b'\n' => escaped.extend(b"\\n"),
            b'\r' => escaped.extend(b"\\r"),
            b'\t' => escaped.extend(b"\\t"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests;
