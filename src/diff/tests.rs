use std::path::Path;

use std::collections::HashMap;

use imara_diff::Token;

use super::{
    Opcode, Tag, grouped_opcodes, intern_line, opcodes_with_limits, push_opcode,
    split_lines_keepends, unified_diff, whole_file_opcodes,
};

#[test]
fn emits_no_output_for_identical_inputs() {
    assert_eq!(unified_diff(Path::new("x.env"), b"A=1\n", b"A=1\n"), b"");
}

#[test]
fn matches_fixture_shape_for_single_line_diff() {
    assert_eq!(
        unified_diff(Path::new("x.env"), b"A=1\n", b"A=2\n"),
        b"--- x.env (before)\n+++ x.env (after)\n@@ -1 +1 @@\n-A=1\n+A=2\n"
    );
}

#[test]
fn preserves_missing_final_newline_shape() {
    assert_eq!(
        unified_diff(Path::new("x.env"), b"A=1", b"A=2"),
        b"--- x.env (before)\n+++ x.env (after)\n@@ -1 +1 @@\n-A=1+A=2"
    );
}

#[test]
fn splits_distant_changes_into_multiple_hunks() {
    assert_eq!(
        unified_diff(
            Path::new("x.env"),
            b"A=1\nB=2\nC=3\nD=4\nE=5\nF=6\nG=7\nH=8\nI=9\n",
            b"A=x\nB=2\nC=3\nD=4\nE=5\nF=6\nG=7\nH=8\nI=x\n",
        ),
        b"--- x.env (before)\n+++ x.env (after)\n@@ -1,4 +1,4 @@\n-A=1\n+A=x\n B=2\n C=3\n D=4\n@@ -6,4 +6,4 @@\n F=6\n G=7\n H=8\n-I=9\n+I=x\n"
    );
}

#[test]
fn formats_pure_insertions_with_zero_length_before_range() {
    assert_eq!(
        unified_diff(Path::new("x.env"), b"", b"A=1\n"),
        b"--- x.env (before)\n+++ x.env (after)\n@@ -0,0 +1 @@\n+A=1\n"
    );
}

#[test]
fn context_reaching_file_start_has_correct_hunk_header() {
    assert_eq!(
        unified_diff(
            Path::new("x.env"),
            b"A=1\nB=2\nC=3\nD=4\nE=5\n",
            b"A=1\nB=x\nC=3\nD=4\nE=5\n",
        ),
        b"--- x.env (before)\n+++ x.env (after)\n@@ -1,5 +1,5 @@\n A=1\n-B=2\n+B=x\n C=3\n D=4\n E=5\n"
    );
}

#[test]
fn large_line_count_diff_stays_small() {
    let mut before = Vec::new();
    let mut after = Vec::new();
    for index in 0..12_000 {
        before.extend(format!("KEY_{index}=same\n").as_bytes());
        if index == 11_000 {
            after.extend(format!("KEY_{index}=changed\n").as_bytes());
        } else {
            after.extend(format!("KEY_{index}=same\n").as_bytes());
        }
    }

    let output = unified_diff(Path::new("x.env"), &before, &after);
    assert!(output.len() < 512, "{}", String::from_utf8_lossy(&output));
    assert!(
        output
            .windows(b"-KEY_11000=same\n".len())
            .any(|window| window == b"-KEY_11000=same\n")
    );
    assert!(
        output
            .windows(b"+KEY_11000=changed\n".len())
            .any(|window| window == b"+KEY_11000=changed\n")
    );
}

#[test]
fn grouping_empty_opcodes_is_empty() {
    assert!(grouped_opcodes(&[], 3).is_empty());
}

#[test]
fn grouping_keeps_equal_only_opcodes_out_of_output_groups() {
    let groups = grouped_opcodes(
        &[Opcode {
            tag: Tag::Equal,
            a_start: 0,
            a_end: 1,
            b_start: 0,
            b_end: 1,
        }],
        3,
    );
    assert!(groups.is_empty());
}

#[test]
fn whole_file_fallback_opcodes_cover_empty_delete_insert_and_replace() {
    assert!(whole_file_opcodes(0, 0).is_empty());

    assert_eq!(
        whole_file_opcodes(2, 0),
        vec![Opcode {
            tag: Tag::Delete,
            a_start: 0,
            a_end: 2,
            b_start: 0,
            b_end: 0,
        }]
    );
    assert_eq!(
        whole_file_opcodes(0, 3),
        vec![Opcode {
            tag: Tag::Insert,
            a_start: 0,
            a_end: 0,
            b_start: 0,
            b_end: 3,
        }]
    );
    assert_eq!(
        whole_file_opcodes(2, 3),
        vec![
            Opcode {
                tag: Tag::Delete,
                a_start: 0,
                a_end: 2,
                b_start: 0,
                b_end: 0,
            },
            Opcode {
                tag: Tag::Insert,
                a_start: 2,
                a_end: 2,
                b_start: 0,
                b_end: 3,
            },
        ]
    );
}

#[test]
fn push_opcode_merges_adjacent_spans_with_the_same_tag() {
    let mut opcodes = Vec::new();

    push_opcode(&mut opcodes, Tag::Equal, 0, 1, 0, 1);
    push_opcode(&mut opcodes, Tag::Equal, 1, 3, 1, 3);
    push_opcode(&mut opcodes, Tag::Insert, 3, 3, 3, 4);

    assert_eq!(
        opcodes,
        vec![
            Opcode {
                tag: Tag::Equal,
                a_start: 0,
                a_end: 3,
                b_start: 0,
                b_end: 3,
            },
            Opcode {
                tag: Tag::Insert,
                a_start: 3,
                a_end: 3,
                b_start: 3,
                b_end: 4,
            },
        ]
    );
}

#[test]
fn intern_line_reuses_tokens_and_rejects_token_overflow() {
    let mut tokens = HashMap::new();
    let mut next_token = 0;

    assert_eq!(
        intern_line(b"A=1\n", &mut tokens, &mut next_token, i32::MAX as u32),
        Some(Token(0))
    );
    assert_eq!(
        intern_line(b"A=1\n", &mut tokens, &mut next_token, i32::MAX as u32),
        Some(Token(0))
    );
    assert_eq!(next_token, 1);

    next_token = i32::MAX as u32;
    assert_eq!(
        intern_line(b"B=2\n", &mut tokens, &mut next_token, i32::MAX as u32),
        None
    );
}

#[test]
fn opcodes_fall_back_to_whole_file_diff_when_imara_limits_are_exceeded() {
    let before = [&b"A=1\n"[..]];
    let after = [&b"B=2\n"[..]];

    assert_eq!(
        opcodes_with_limits(&before, &after, 1, i32::MAX as u32),
        whole_file_opcodes(1, 1)
    );
    assert_eq!(
        opcodes_with_limits(&before, &after, i32::MAX as usize, 1),
        whole_file_opcodes(1, 1)
    );
}

#[test]
fn split_lines_keepends_handles_lf_crlf_bare_cr_and_unterminated_tail() {
    assert_eq!(
        split_lines_keepends(b"A=1\nB=2\r\nC=3\rD=4"),
        vec![&b"A=1\n"[..], &b"B=2\r\n"[..], &b"C=3\r"[..], &b"D=4"[..]]
    );
}
