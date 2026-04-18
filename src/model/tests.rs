use super::{BlankLine, CommentLine, InvalidLine, Line};

#[test]
fn line_helpers_cover_preserved_line_variants() {
    let blank = Line::Blank(BlankLine {
        text: b"  ".to_vec(),
        newline: b"\n".to_vec(),
    });
    let comment = Line::Comment(CommentLine {
        text: b"# comment".to_vec(),
        newline: b"\r\n".to_vec(),
    });
    let invalid = Line::Invalid(InvalidLine {
        text: b"not valid".to_vec(),
        newline: Vec::new(),
    });

    assert_eq!(blank.kind(), "blank");
    assert_eq!(comment.kind(), "comment");
    assert_eq!(invalid.kind(), "invalid");
    assert_eq!(blank.newline(), b"\n");
    assert_eq!(comment.newline(), b"\r\n");
    assert_eq!(invalid.newline(), b"");
    assert_eq!(blank.with_newline(b"\r".to_vec()).newline(), b"\r");
    assert_eq!(comment.with_newline(b"\n".to_vec()).newline(), b"\n");
    assert_eq!(invalid.with_newline(b"\r\n".to_vec()).newline(), b"\r\n");
}
