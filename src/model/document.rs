use super::Line;

/// Parsed `.env` document plus the newline style to use for new lines.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Document {
    /// Physical lines in source order.
    pub lines: Vec<Line>,
    /// First observed line terminator, or the platform default for empty files.
    pub preferred_newline: Vec<u8>,
}
