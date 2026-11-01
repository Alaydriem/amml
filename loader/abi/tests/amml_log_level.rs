use amml_abi::{AmmlLogLevel, KnownLogLevel};

#[test]
fn defined_levels_decode_and_other_values_do_not() {
    assert_eq!(AmmlLogLevel::WARN.known(), Some(KnownLogLevel::Warn));
    assert_eq!(AmmlLogLevel(200).known(), None);
}
