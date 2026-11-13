use amml_gdk::PackageContext;

#[test]
fn a_test_process_is_not_packaged() {
    assert!(!PackageContext::new().is_packaged());
}
