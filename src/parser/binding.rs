use crate::diagnostics::{is_key_continue, is_key_start};
use crate::model::{BindingLine, BlankLine, CommentLine, InvalidLine, Line, QuoteType};

use super::common::{consume_horizontal_whitespace, is_horizontal_whitespace, lstrip_horizontal};
use super::quoted::parse_quoted_binding;

/// Classifies a physical line without interpreting unsupported syntax.
pub(super) fn parse_line(text: &[u8], newline: &[u8]) -> Line {
    if text.iter().all(u8::is_ascii_whitespace) {
        return Line::Blank(BlankLine {
            text: text.to_vec(),
            newline: newline.to_vec(),
        });
    }

    if lstrip_horizontal(text).starts_with(b"#") {
        return Line::Comment(CommentLine {
            text: text.to_vec(),
            newline: newline.to_vec(),
        });
    }

    parse_binding(text, newline).map_or_else(
        || {
            Line::Invalid(InvalidLine {
                text: text.to_vec(),
                newline: newline.to_vec(),
            })
        },
        Line::Binding,
    )
}

fn parse_binding(text: &[u8], newline: &[u8]) -> Option<BindingLine> {
    let mut index = 0;
    let mut export = false;

    if text.starts_with(b"export")
        && text.len() > b"export".len()
        && is_horizontal_whitespace(text[b"export".len()])
    {
        export = true;
        index = consume_horizontal_whitespace(text, b"export".len());
    }

    let key_start = index;
    if index >= text.len() || !is_key_start(text[index]) {
        return None;
    }
    index += 1;
    while index < text.len() && is_key_continue(text[index]) {
        index += 1;
    }

    let key = std::str::from_utf8(&text[key_start..index]).ok()?;

    index = consume_horizontal_whitespace(text, index);
    if index >= text.len() || text[index] != b'=' {
        return None;
    }

    index += 1;
    index = consume_horizontal_whitespace(text, index);
    // Prefix is preserved so edits keep existing `export`, key, `=`, and spacing.
    let prefix = text[..index].to_vec();

    if index < text.len() && matches!(text[index], b'\'' | b'"') {
        return parse_quoted_binding(text, newline, export, key, prefix, index);
    }

    let (raw_value, suffix, inline_comment) = split_unquoted_value_and_suffix(&text[index..]);
    Some(BindingLine {
        export,
        key: key.to_owned(),
        value: raw_value.clone(),
        raw_value,
        quote_type: QuoteType::None,
        prefix,
        suffix,
        inline_comment,
        original_text: text.to_vec(),
        newline: newline.to_vec(),
    })
}

fn split_unquoted_value_and_suffix(value_text: &[u8]) -> (Vec<u8>, Vec<u8>, Option<Vec<u8>>) {
    let Some(suffix_start) = find_unquoted_comment_suffix_start(value_text) else {
        return (value_text.to_vec(), Vec::new(), None);
    };

    let raw_value = value_text[..suffix_start].to_vec();
    let suffix = value_text[suffix_start..].to_vec();
    (raw_value, suffix.clone(), Some(suffix))
}

fn find_unquoted_comment_suffix_start(value_text: &[u8]) -> Option<usize> {
    for (index, byte) in value_text.iter().enumerate() {
        // Only ` #` starts an inline comment. `#` at the beginning of a value or
        // after non-whitespace stays part of the value.
        if *byte != b'#'
            || index == 0
            || !is_horizontal_whitespace(value_text[index.saturating_sub(1)])
        {
            continue;
        }

        let mut suffix_start = index - 1;
        while suffix_start > 0 && is_horizontal_whitespace(value_text[suffix_start - 1]) {
            suffix_start -= 1;
        }
        return Some(suffix_start);
    }
    None
}
