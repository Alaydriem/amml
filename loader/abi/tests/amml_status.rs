use amml_abi::AmmlStatus;

#[test]
fn only_ok_is_ok_including_values_this_version_does_not_define() {
    assert!(AmmlStatus::OK.is_ok());
    assert!(!AmmlStatus::FAILED.is_ok());
    assert!(!AmmlStatus(-1).is_ok());
    assert!(!AmmlStatus(1000).is_ok());
}
