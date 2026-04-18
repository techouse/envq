use std::path::Path;

use super::{Opcode, Tag, grouped_opcodes, unified_diff};

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
