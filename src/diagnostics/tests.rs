use super::is_valid_key;

#[test]
fn key_validation() {
    assert!(is_valid_key("KEY"));
    assert!(is_valid_key("_KEY_1"));
    assert!(!is_valid_key("1KEY"));
    assert!(!is_valid_key("KEY-NAME"));
    assert!(!is_valid_key(""));
}
