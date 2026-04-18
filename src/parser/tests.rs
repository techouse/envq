use crate::model::{Line, QuoteType};
use crate::render::render_document;

use super::parse_document;

#[test]
fn classifies_lines_and_renders_losslessly() {
    let source = b"\n  \n# comment\n  # indented\nKEY=value\nbad line\n";
    let document = parse_document(source);

    assert!(matches!(document.lines[0], Line::Blank(_)));
    assert!(matches!(document.lines[1], Line::Blank(_)));
    assert!(matches!(document.lines[2], Line::Comment(_)));
    assert!(matches!(document.lines[3], Line::Comment(_)));
    assert!(matches!(document.lines[4], Line::Binding(_)));
    assert!(matches!(document.lines[5], Line::Invalid(_)));
    assert_eq!(render_document(&document), source);
}

#[test]
fn preserves_export_spacing_and_inline_comment() {
    let document = parse_document(b"export KEY = value # keep\n");
    let Line::Binding(line) = &document.lines[0] else {
        panic!("expected binding");
    };

    assert!(line.export);
    assert_eq!(line.key, "KEY");
    assert_eq!(line.value, b"value");
    assert_eq!(line.raw_value, b"value");
    assert_eq!(line.prefix, b"export KEY = ");
    assert_eq!(line.suffix, b" # keep");
    assert_eq!(line.inline_comment.as_deref(), Some(&b" # keep"[..]));
}

#[test]
fn parses_hash_ambiguity_and_quoted_values() {
    let document = parse_document(b"A=a#b\nB=#c\nC=\"a\\n\\t\"\nD='one two'\n");
    let Line::Binding(first) = &document.lines[0] else {
        panic!("expected binding");
    };
    let Line::Binding(second) = &document.lines[1] else {
        panic!("expected binding");
    };
    let Line::Binding(third) = &document.lines[2] else {
        panic!("expected binding");
    };
    let Line::Binding(fourth) = &document.lines[3] else {
        panic!("expected binding");
    };

    assert_eq!(first.value, b"a#b");
    assert_eq!(second.value, b"#c");
    assert_eq!(third.value, b"a\n\t");
    assert_eq!(third.quote_type, QuoteType::Double);
    assert_eq!(fourth.value, b"one two");
    assert_eq!(fourth.quote_type, QuoteType::Single);
}

#[test]
fn invalid_quoted_suffix_is_invalid() {
    let document = parse_document(
        br#"A="value"suffix
"#,
    );
    assert!(matches!(document.lines[0], Line::Invalid(_)));
    assert_eq!(
        render_document(&document),
        br#"A="value"suffix
"#
    );
}

#[test]
fn quoted_values_can_have_inline_comments_and_missing_closers() {
    let document = parse_document(
        br##"A="value" # keep
B='a\b' # keep
C="hash"#keep
D="unterminated
"##,
    );

    let Line::Binding(first) = &document.lines[0] else {
        panic!("expected binding");
    };
    let Line::Binding(second) = &document.lines[1] else {
        panic!("expected binding");
    };

    assert_eq!(first.inline_comment.as_deref(), Some(&b" # keep"[..]));
    assert_eq!(second.value, br"a\b");
    assert_eq!(second.inline_comment.as_deref(), Some(&b" # keep"[..]));
    let Line::Binding(third) = &document.lines[2] else {
        panic!("expected binding");
    };

    assert_eq!(third.inline_comment.as_deref(), Some(&b"#keep"[..]));
    assert!(matches!(document.lines[3], Line::Invalid(_)));
}

#[test]
fn tracks_newline_styles() {
    let document = parse_document(b"A=1\r\nB=2\rC=3\nD=4");
    assert_eq!(document.preferred_newline, b"\r\n");
    assert_eq!(document.lines[0].newline(), b"\r\n");
    assert_eq!(document.lines[1].newline(), b"\r");
    assert_eq!(document.lines[2].newline(), b"\n");
    assert_eq!(document.lines[3].newline(), b"");
}

#[test]
fn decodes_internal_trailing_backslash_literal() {
    assert_eq!(super::quoted::decode_double_quoted(b"abc\\"), b"abc\\");
}

#[test]
fn decodes_all_supported_and_unknown_double_quote_escapes() {
    assert_eq!(
        super::quoted::decode_double_quoted(br#"a\r\\\"x\q"#),
        b"a\r\\\"x\\q"
    );
}
