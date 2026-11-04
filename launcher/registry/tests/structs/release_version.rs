use amml_registry::{RegistryError, ReleaseVersion};

#[test]
fn orders_numerically_and_rejects_junk() {
    let high = ReleaseVersion::parse("26.60.10").unwrap();
    let low = ReleaseVersion::parse("26.60.9").unwrap();
    assert!(high > low);

    for junk in ["26.60", "v26.60.1", "26.60.1-rc1", "26.60.1.2", "26..1", ""] {
        match ReleaseVersion::parse(junk) {
            Err(RegistryError::BadVersion(raw)) => assert_eq!(raw, junk),
            other => panic!("`{junk}` gave {other:?}"),
        }
    }
}
