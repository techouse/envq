use super::QuoteType;

/// A blank physical line, preserving original bytes and line terminator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlankLine {
    /// Line body without the terminator.
    pub text: Vec<u8>,
    /// Original line terminator, if any.
    pub newline: Vec<u8>,
}

/// A full-line comment, preserving original bytes and line terminator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommentLine {
    /// Line body without the terminator.
    pub text: Vec<u8>,
    /// Original line terminator, if any.
    pub newline: Vec<u8>,
}

/// Unsupported or malformed syntax preserved byte-for-byte.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidLine {
    /// Line body without the terminator.
    pub text: Vec<u8>,
    /// Original line terminator, if any.
    pub newline: Vec<u8>,
}

/// A supported key/value binding with enough raw pieces to render losslessly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingLine {
    /// Whether the original binding used the supported `export KEY=...` prefix.
    pub export: bool,
    /// ASCII key name.
    pub key: String,
    /// Resolved value bytes after quote handling.
    pub value: Vec<u8>,
    /// Original or newly encoded value token, including quotes when present.
    pub raw_value: Vec<u8>,
    /// Quote form used by `raw_value`.
    pub quote_type: QuoteType,
    /// Bytes from the start of the line through the spacing after `=`.
    pub prefix: Vec<u8>,
    /// Bytes after the value token, including any inline comment.
    pub suffix: Vec<u8>,
    /// Inline comment bytes when the parser can identify them unambiguously.
    pub inline_comment: Option<Vec<u8>>,
    /// Original physical line body without the line terminator.
    pub original_text: Vec<u8>,
    /// Original line terminator, if any.
    pub newline: Vec<u8>,
}

/// Parsed physical line classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Line {
    /// Whitespace-only line.
    Blank(BlankLine),
    /// Full-line comment.
    Comment(CommentLine),
    /// Supported key/value binding.
    Binding(BindingLine),
    /// Unsupported syntax preserved without interpretation.
    Invalid(InvalidLine),
}

impl Line {
    /// Stable lowercase kind name used by tests and fixtures.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Line::Blank(_) => "blank",
            Line::Comment(_) => "comment",
            Line::Binding(_) => "binding",
            Line::Invalid(_) => "invalid",
        }
    }

    /// Returns this line's original or assigned line terminator.
    #[must_use]
    pub fn newline(&self) -> &[u8] {
        match self {
            Line::Blank(line) => &line.newline,
            Line::Comment(line) => &line.newline,
            Line::Binding(line) => &line.newline,
            Line::Invalid(line) => &line.newline,
        }
    }

    /// Returns a clone of this line with a different line terminator.
    #[must_use]
    pub fn with_newline(&self, newline: Vec<u8>) -> Self {
        match self {
            Line::Blank(line) => Line::Blank(BlankLine {
                text: line.text.clone(),
                newline,
            }),
            Line::Comment(line) => Line::Comment(CommentLine {
                text: line.text.clone(),
                newline,
            }),
            Line::Binding(line) => Line::Binding(BindingLine {
                export: line.export,
                key: line.key.clone(),
                value: line.value.clone(),
                raw_value: line.raw_value.clone(),
                quote_type: line.quote_type,
                prefix: line.prefix.clone(),
                suffix: line.suffix.clone(),
                inline_comment: line.inline_comment.clone(),
                original_text: line.original_text.clone(),
                newline,
            }),
            Line::Invalid(line) => Line::Invalid(InvalidLine {
                text: line.text.clone(),
                newline,
            }),
        }
    }
}
