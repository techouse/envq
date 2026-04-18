use crate::parser::parse_document;
use crate::render::render_document;

use super::{get_value, has_key, list_bindings, set_value, unset_key};

fn default_newline() -> &'static [u8] {
    if cfg!(windows) { b"\r\n" } else { b"\n" }
}

#[test]
fn duplicate_key_semantics() {
    let document = parse_document(b"A=1\nA=2\nB=3\n");

    assert_eq!(get_value(&document, "A"), Some(&b"1"[..]));
    assert!(has_key(&document, "A"));
    assert_eq!(
        list_bindings(&document),
        vec![
            ("A".to_owned(), b"1".to_vec()),
            ("A".to_owned(), b"2".to_vec()),
            ("B".to_owned(), b"3".to_vec()),
        ]
    );
    assert_eq!(
        render_document(&set_value(&document, "A", b"x")),
        b"A=x\nA=2\nB=3\n"
    );

    let unset_result = unset_key(&document, "A");
    assert_eq!(unset_result.removed_count, 2);
    assert_eq!(render_document(&unset_result.document), b"B=3\n");
}

#[test]
fn preserves_prefix_spacing_comments_and_unambiguous_suffixes() {
    let document = parse_document(b"export A = old # keep\n");
    assert_eq!(
        render_document(&set_value(&document, "A", b"new")),
        b"export A = new # keep\n"
    );

    let document = parse_document(b"A = old # keep\n");
    assert_eq!(
        render_document(&set_value(&document, "A", b"")),
        br#"A = "" # keep
"#
    );

    let document = parse_document(
        br#"A="old"   
"#,
    );
    assert_eq!(
        render_document(&set_value(&document, "A", b"new")),
        br#"A="new"   
"#
    );
}

#[test]
fn appends_and_repairs_missing_terminal_newline() {
    let document = parse_document(b"A=1");
    let mut expected = b"A=1".to_vec();
    expected.extend(default_newline());
    expected.extend(b"B=\"two words\"");
    expected.extend(default_newline());

    assert_eq!(
        render_document(&set_value(&document, "B", b"two words")),
        expected
    );
}

#[test]
fn unset_can_empty_file() {
    let document = parse_document(b"A=1\nA=2");
    let unset_result = unset_key(&document, "A");

    assert_eq!(unset_result.removed_count, 2);
    assert_eq!(render_document(&unset_result.document), b"");
}
