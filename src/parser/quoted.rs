use crate::model::{BindingLine, QuoteType};

use super::common::{lstrip_horizontal, trim_horizontal};

/// Parses a single- or double-quoted binding once the opening quote is known.
pub(super) fn parse_quoted_binding(
    text: &[u8],
    newline: &[u8],
    export: bool,
    key: &str,
    prefix: Vec<u8>,
    value_start: usize,
) -> Option<BindingLine> {
    let quote = text[value_start];
    let closing_quote = find_closing_quote(text, value_start + 1, quote)?;
    let raw_value = text[value_start..=closing_quote].to_vec();
    let suffix = text[closing_quote + 1..].to_vec();
    if !is_valid_quoted_suffix(&suffix) {
        return None;
    }

    let quote_type = if quote == b'\'' {
        QuoteType::Single
    } else {
        QuoteType::Double
    };
    let inner_value = &text[value_start + 1..closing_quote];
    let value = if quote_type == QuoteType::Single {
        inner_value.to_vec()
    } else {
        decode_double_quoted(inner_value)
    };
    let has_inline_comment = lstrip_horizontal(&suffix).starts_with(b"#");
    let inline_comment = if has_inline_comment {
        Some(suffix.clone())
    } else {
        None
    };

    Some(BindingLine {
        export,
        key: key.to_owned(),
        value,
        raw_value,
        quote_type,
        prefix,
        suffix,
        inline_comment,
        original_text: text.to_vec(),
        newline: newline.to_vec(),
    })
}

fn find_closing_quote(text: &[u8], mut index: usize, quote: u8) -> Option<usize> {
    while index < text.len() {
        let byte = text[index];
        // Backslash only escapes characters inside double quotes.
        if quote == b'"' && byte == b'\\' {
            index += 2;
            continue;
        }
        if byte == quote {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn is_valid_quoted_suffix(suffix: &[u8]) -> bool {
    if trim_horizontal(suffix).is_empty() {
        return true;
    }
    lstrip_horizontal(suffix).starts_with(b"#")
}

pub(super) fn decode_double_quoted(value: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(value.len());
    let mut index = 0;

    while index < value.len() {
        let byte = value[index];
        if byte != b'\\' {
            decoded.push(byte);
            index += 1;
            continue;
        }

        if index + 1 >= value.len() {
            decoded.push(b'\\');
            index += 1;
            continue;
        }

        match value[index + 1] {
            b'n' => decoded.push(b'\n'),
            b'r' => decoded.push(b'\r'),
            b't' => decoded.push(b'\t'),
            b'\\' => decoded.push(b'\\'),
            b'"' => decoded.push(b'"'),
            escaped => {
                // Unknown escapes stay literal, including the backslash.
                decoded.push(b'\\');
                decoded.push(escaped);
            }
        }
        index += 2;
    }

    decoded
}
