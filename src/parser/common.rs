/// Advances over spaces and tabs only.
pub(super) fn consume_horizontal_whitespace(text: &[u8], mut index: usize) -> usize {
    while index < text.len() && is_horizontal_whitespace(text[index]) {
        index += 1;
    }
    index
}

/// Trims spaces and tabs from the left side.
pub(super) fn lstrip_horizontal(text: &[u8]) -> &[u8] {
    let start = consume_horizontal_whitespace(text, 0);
    &text[start..]
}

/// Trims spaces and tabs from both sides.
pub(super) fn trim_horizontal(text: &[u8]) -> &[u8] {
    let start = text
        .iter()
        .position(|byte| !is_horizontal_whitespace(*byte))
        .unwrap_or(text.len());
    let end = text
        .iter()
        .rposition(|byte| !is_horizontal_whitespace(*byte))
        .map_or(start, |index| index + 1);
    &text[start..end]
}

/// Horizontal whitespace recognized by the `.env` grammar.
pub(super) const fn is_horizontal_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t')
}
