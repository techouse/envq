use super::encode_value_auto;

#[test]
fn auto_quoting() {
    let cases: &[(&[u8], &[u8])] = &[
        (b"", b""),
        (b"simple-1", b"simple-1"),
        (b"two words", br#""two words""#),
        (b"a#b", br#""a#b""#),
        (b"a\"b\\c\n", br#""a\"b\\c\n""#),
        (b"carriage\rreturn", br#""carriage\rreturn""#),
        (b"tab\tvalue", br#""tab\tvalue""#),
        (b"\xff", b"\xff"),
    ];

    for (value, expected) in cases {
        assert_eq!(encode_value_auto(value).text, *expected);
    }
}
