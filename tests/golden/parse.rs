use envq::editor::list_bindings;
use envq::parser::parse_document;
use envq::render::render_document;

use crate::support::{
    case_id, cases, content_bytes, expected_bytes, load_manifest, object, platform_applies,
    required, string_list, string_pairs,
};

#[test]
fn parse_golden_fixtures() {
    let manifest = load_manifest("parse.json");
    for case in cases(&manifest) {
        if !platform_applies(case) {
            continue;
        }

        let input_spec = object(required(case, "input"));
        let source_bytes = content_bytes(input_spec);
        let document = parse_document(&source_bytes);
        let expect = object(required(case, "expect"));

        let expected_bindings = string_pairs(required(expect, "bindings"));
        assert_eq!(
            list_bindings(&document),
            expected_bindings,
            "{}",
            case_id(case)
        );

        let expected_line_kinds = string_list(required(expect, "line_kinds"));
        let actual_line_kinds = document
            .lines
            .iter()
            .map(|line| line.kind().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(actual_line_kinds, expected_line_kinds, "{}", case_id(case));

        let expected_rendered = expected_bytes(
            object(required(expect, "rendered")),
            Some(source_bytes.as_slice()),
        );
        assert_eq!(
            render_document(&document),
            expected_rendered,
            "{}",
            case_id(case)
        );
    }
}
