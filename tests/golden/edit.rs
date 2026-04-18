use envq::editor::{set_value, unset_key};
use envq::parser::parse_document;
use envq::render::render_document;

use crate::support::{
    case_id, cases, content_bytes, expected_bytes, load_manifest, object, platform_applies,
    required, required_str,
};

#[test]
fn edit_golden_fixtures() {
    let manifest = load_manifest("edit.json");
    for case in cases(&manifest) {
        if !platform_applies(case) {
            continue;
        }

        let input_spec = object(required(case, "input"));
        let source_bytes = content_bytes(input_spec);
        let document = parse_document(&source_bytes);
        let operation = object(required(case, "operation"));
        let operation_name = required_str(operation, "name");
        let key = required_str(operation, "key");
        let expect = object(required(case, "expect"));

        let updated = match operation_name {
            "set" => {
                let value = required_str(operation, "value").as_bytes().to_vec();
                set_value(&document, key, &value)
            }
            "clear" => set_value(&document, key, b""),
            "unset" => {
                let unset_result = unset_key(&document, key);
                let removed_count = required(expect, "removed_count")
                    .as_u64()
                    .expect("removed_count must be integer")
                    as usize;
                assert_eq!(
                    unset_result.removed_count,
                    removed_count,
                    "{}",
                    case_id(case)
                );
                unset_result.document
            }
            other => panic!("unsupported edit operation: {other}"),
        };

        let expected_output = expected_bytes(
            object(required(expect, "output")),
            Some(source_bytes.as_slice()),
        );
        assert_eq!(
            render_document(&updated),
            expected_output,
            "{}",
            case_id(case)
        );
    }
}
