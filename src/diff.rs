//! Small unified-diff implementation matching envq's fixture shape.
//!
//! The output is byte-oriented and intentionally omits a "no newline at EOF"
//! marker, matching the behavior captured by the golden tests.

use std::collections::HashMap;
use std::path::Path;

use imara_diff::{Algorithm, Diff, Token};

const MAX_IMARA_SEQUENCE_LEN: usize = i32::MAX as usize;
const MAX_IMARA_TOKEN_COUNT: u32 = i32::MAX as u32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tag {
    Equal,
    Delete,
    Insert,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Opcode {
    tag: Tag,
    a_start: usize,
    a_end: usize,
    b_start: usize,
    b_end: usize,
}

/// Emits a unified diff for one file path.
pub(crate) fn unified_diff(path: &Path, before: &[u8], after: &[u8]) -> Vec<u8> {
    if before == after {
        return Vec::new();
    }

    let before_lines = split_lines_keepends(before);
    let after_lines = split_lines_keepends(after);
    let opcodes = opcodes(&before_lines, &after_lines);
    let groups = grouped_opcodes(&opcodes, 3);

    let mut output = Vec::new();
    output.extend(format!("--- {} (before)\n", path.display()).as_bytes());
    output.extend(format!("+++ {} (after)\n", path.display()).as_bytes());

    for group in groups {
        let first = &group[0];
        let last = &group[group.len() - 1];
        output.extend(
            format!(
                "@@ -{} +{} @@\n",
                format_range_unified(first.a_start, last.a_end),
                format_range_unified(first.b_start, last.b_end)
            )
            .as_bytes(),
        );

        for opcode in group {
            match opcode.tag {
                Tag::Equal => {
                    for line in &before_lines[opcode.a_start..opcode.a_end] {
                        output.push(b' ');
                        output.extend(*line);
                    }
                }
                Tag::Delete => {
                    for line in &before_lines[opcode.a_start..opcode.a_end] {
                        output.push(b'-');
                        output.extend(*line);
                    }
                }
                Tag::Insert => {
                    for line in &after_lines[opcode.b_start..opcode.b_end] {
                        output.push(b'+');
                        output.extend(*line);
                    }
                }
            }
        }
    }

    output
}

fn opcodes(before: &[&[u8]], after: &[&[u8]]) -> Vec<Opcode> {
    opcodes_with_limits(before, after, MAX_IMARA_SEQUENCE_LEN, MAX_IMARA_TOKEN_COUNT)
}

fn opcodes_with_limits(
    before: &[&[u8]],
    after: &[&[u8]],
    max_sequence_len: usize,
    max_token_count: u32,
) -> Vec<Opcode> {
    let Some((before_tokens, after_tokens, token_count)) =
        intern_lines(before, after, max_sequence_len, max_token_count)
    else {
        return whole_file_opcodes(before.len(), after.len());
    };

    let mut diff = Diff::default();
    diff.compute_with(Algorithm::Myers, &before_tokens, &after_tokens, token_count);
    opcodes_from_hunks(diff.hunks(), before.len(), after.len())
}

fn intern_lines(
    before: &[&[u8]],
    after: &[&[u8]],
    max_sequence_len: usize,
    max_token_count: u32,
) -> Option<(Vec<Token>, Vec<Token>, u32)> {
    // `imara-diff` supports less than i32::MAX tokens on each side. The guard
    // keeps CLI diff generation from panicking on extreme inputs.
    if before.len() >= max_sequence_len || after.len() >= max_sequence_len {
        return None;
    }

    let mut tokens = HashMap::new();
    let mut next_token = 0;
    let before_tokens = intern_line_slice(before, &mut tokens, &mut next_token, max_token_count)?;
    let after_tokens = intern_line_slice(after, &mut tokens, &mut next_token, max_token_count)?;
    Some((before_tokens, after_tokens, next_token))
}

fn intern_line_slice(
    lines: &[&[u8]],
    tokens: &mut HashMap<Vec<u8>, Token>,
    next_token: &mut u32,
    max_token_count: u32,
) -> Option<Vec<Token>> {
    lines
        .iter()
        .map(|line| intern_line(line, tokens, next_token, max_token_count))
        .collect()
}

fn intern_line(
    line: &[u8],
    tokens: &mut HashMap<Vec<u8>, Token>,
    next_token: &mut u32,
    max_token_count: u32,
) -> Option<Token> {
    if let Some(token) = tokens.get(line) {
        return Some(*token);
    }
    if *next_token >= max_token_count {
        return None;
    }

    let token = Token(*next_token);
    *next_token += 1;
    tokens.insert(line.to_vec(), token);
    Some(token)
}

fn opcodes_from_hunks(
    hunks: impl IntoIterator<Item = imara_diff::Hunk>,
    before_len: usize,
    after_len: usize,
) -> Vec<Opcode> {
    let mut codes = Vec::new();
    let mut before_index = 0;
    let mut after_index = 0;

    for hunk in hunks {
        let a_start = hunk.before.start as usize;
        let a_end = hunk.before.end as usize;
        let b_start = hunk.after.start as usize;
        let b_end = hunk.after.end as usize;

        if before_index < a_start || after_index < b_start {
            push_opcode(
                &mut codes,
                Tag::Equal,
                before_index,
                a_start,
                after_index,
                b_start,
            );
        }
        if a_start < a_end {
            push_opcode(&mut codes, Tag::Delete, a_start, a_end, b_start, b_start);
        }
        if b_start < b_end {
            push_opcode(&mut codes, Tag::Insert, a_end, a_end, b_start, b_end);
        }

        before_index = a_end;
        after_index = b_end;
    }

    if before_index < before_len || after_index < after_len {
        push_opcode(
            &mut codes,
            Tag::Equal,
            before_index,
            before_len,
            after_index,
            after_len,
        );
    }

    codes
}

fn whole_file_opcodes(before_len: usize, after_len: usize) -> Vec<Opcode> {
    let mut codes = Vec::new();
    if before_len > 0 {
        push_opcode(&mut codes, Tag::Delete, 0, before_len, 0, 0);
    }
    if after_len > 0 {
        push_opcode(
            &mut codes,
            Tag::Insert,
            before_len,
            before_len,
            0,
            after_len,
        );
    }
    codes
}

fn push_opcode(
    codes: &mut Vec<Opcode>,
    tag: Tag,
    a_start: usize,
    a_end: usize,
    b_start: usize,
    b_end: usize,
) {
    if let Some(last) = codes.last_mut()
        && last.tag == tag
        && last.a_end == a_start
        && last.b_end == b_start
    {
        last.a_end = a_end;
        last.b_end = b_end;
        return;
    }

    codes.push(Opcode {
        tag,
        a_start,
        a_end,
        b_start,
        b_end,
    });
}

fn grouped_opcodes(opcodes: &[Opcode], context: usize) -> Vec<Vec<Opcode>> {
    if opcodes.is_empty() {
        return Vec::new();
    }

    let mut codes = opcodes.to_vec();
    if let Some(first) = codes.first_mut()
        && first.tag == Tag::Equal
    {
        first.a_start = first.a_start.max(first.a_end.saturating_sub(context));
        first.b_start = first.b_start.max(first.b_end.saturating_sub(context));
    }
    if let Some(last) = codes.last_mut()
        && last.tag == Tag::Equal
    {
        last.a_end = last.a_end.min(last.a_start + context);
        last.b_end = last.b_end.min(last.b_start + context);
    }

    let mut groups = Vec::new();
    let mut group = Vec::new();
    let split_context = context * 2;

    for mut code in codes {
        // Large equal blocks split hunks while retaining context on both sides.
        if code.tag == Tag::Equal && code.a_end - code.a_start > split_context {
            group.push(Opcode {
                tag: Tag::Equal,
                a_start: code.a_start,
                a_end: code.a_start + context,
                b_start: code.b_start,
                b_end: code.b_start + context,
            });
            groups.push(group);
            group = Vec::new();
            code.a_start = code.a_end - context;
            code.b_start = code.b_end - context;
        }
        group.push(code);
    }

    if group
        .iter()
        .any(|code| matches!(code.tag, Tag::Delete | Tag::Insert))
    {
        groups.push(group);
    }

    groups
}

fn format_range_unified(start: usize, stop: usize) -> String {
    let mut beginning = start + 1;
    let length = stop - start;
    if length == 1 {
        return beginning.to_string();
    }
    if length == 0 {
        beginning -= 1;
    }
    format!("{beginning},{length}")
}

fn split_lines_keepends(text: &[u8]) -> Vec<&[u8]> {
    // Keep line endings attached for LF, CRLF, and bare CR inputs.
    let mut lines = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while index < text.len() {
        match text[index] {
            b'\r' if index + 1 < text.len() && text[index + 1] == b'\n' => {
                lines.push(&text[start..index + 2]);
                index += 2;
                start = index;
            }
            b'\r' | b'\n' => {
                lines.push(&text[start..index + 1]);
                index += 1;
                start = index;
            }
            _ => index += 1,
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

#[cfg(test)]
mod tests;
