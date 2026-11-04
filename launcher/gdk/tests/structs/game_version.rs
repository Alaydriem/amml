use amml_gdk::{GameVersion, GdkError};

fn v(raw: &str) -> GameVersion {
    GameVersion::parse(raw).unwrap()
}

#[test]
fn zero_padded_and_plain_are_equal() {
    assert_eq!(v("1.26.60.05"), v("1.26.60.5"));
}

#[test]
fn orders_numerically() {
    assert!(v("1.26.100.1") > v("1.26.60.5"));
    assert!(v("1.26.60.5") > v("1.21.120.4"));
}

#[test]
fn instance_name_pads_last_part() {
    assert_eq!(v("1.26.60.5").instance_name(), "1.26.60.05");
    assert_eq!(v("1.26.70.22").instance_name(), "1.26.70.22");
}

#[test]
fn from_package_version_unpacks_third_part() {
    assert_eq!(GameVersion::from_package_version("1.26.6005.0").unwrap(), v("1.26.60.5"));
    assert_eq!(GameVersion::from_package_version("1.26.5203.0").unwrap(), v("1.26.52.3"));
    assert_eq!(GameVersion::from_package_version("1.21.13101.0").unwrap(), v("1.21.131.1"));
}

#[test]
fn rejects_junk() {
    for raw in ["1.26", "a.b.c.d", "1.26.60.5.1", ""] {
        assert!(matches!(GameVersion::parse(raw), Err(GdkError::BadVersion(_))), "{raw}");
    }
}
