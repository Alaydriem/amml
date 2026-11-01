use amml::Boundary;
use amml_abi::AmmlStatus;

#[test]
fn a_panic_becomes_the_fallback_and_the_status_panicked() {
    assert_eq!(Boundary::run(7, || -> i32 { panic!("boom") }), 7);
    assert_eq!(Boundary::guard(|| panic!("boom")), AmmlStatus::PANICKED);
    assert_eq!(Boundary::guard(|| AmmlStatus::MISSING_ENTRY), AmmlStatus::MISSING_ENTRY);
}
