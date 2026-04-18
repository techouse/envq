//! Byte-preserving parser for envq's documented `.env` subset.
//!
//! This parser is intentionally local instead of using a dotenv crate. envq is
//! an editor, not just a loader: it must preserve unsupported syntax, comments,
//! spacing, mixed line endings, duplicate bindings, and invalid UTF-8 bytes so
//! unrelated edits render byte-for-byte. Most dotenv parsers normalize input
//! into UTF-8 key/value maps and implement broader shell-like grammars, which
//! would break envq's fixture-backed compatibility contract.

mod binding;
mod common;
mod newline;
mod quoted;

use crate::model::Document;

use self::binding::parse_line;
use self::newline::{preferred_newline, split_physical_lines};

/// Parses raw file bytes into a byte-backed document model.
#[must_use]
pub fn parse_document(text: &[u8]) -> Document {
    let segments = split_physical_lines(text);
    let preferred_newline = preferred_newline(&segments);
    let lines = segments
        .iter()
        .map(|(line_text, newline)| parse_line(line_text, newline))
        .collect();
    Document {
        lines,
        preferred_newline,
    }
}

#[cfg(test)]
mod tests;
