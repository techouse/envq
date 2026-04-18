use envq::model::ExitCode;

#[test]
fn exit_codes_match_contract() {
    assert_eq!(ExitCode::Success.code(), 0);
    assert_eq!(ExitCode::GeneralError.code(), 1);
    assert_eq!(ExitCode::KeyNotFound.code(), 2);
    assert_eq!(ExitCode::ValidationError.code(), 3);
    assert_eq!(ExitCode::WouldChange.code(), 4);
}
