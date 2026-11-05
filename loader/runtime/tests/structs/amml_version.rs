use amml_runtime::AmmlVersion;

fn version(major: u32, minor: u32, patch: u32) -> AmmlVersion {
    AmmlVersion { major, minor, patch }
}

#[test]
fn parses_three_parts() {
    assert_eq!(AmmlVersion::parse("26.62.0"), Some(version(26, 62, 0)));
}

#[test]
fn drops_a_build_suffix() {
    assert_eq!(AmmlVersion::parse("26.62.0+abc"), Some(version(26, 62, 0)));
}

#[test]
fn a_bare_plus_is_dropped() {
    assert_eq!(AmmlVersion::parse("26.62.0+"), Some(version(26, 62, 0)));
}

#[test]
fn refuses_a_part_over_u32() {
    assert_eq!(AmmlVersion::parse("4294967296.0.0"), None);
    assert_eq!(AmmlVersion::parse("4294967295.0.0"), Some(version(4294967295, 0, 0)));
}

#[test]
fn refuses_bad_shapes() {
    for bad in ["", "26.62", "26.62.0.1", "v26.62.0", "26.062.0", "26..0", "26.62.-1", "+26.62.0", " 26.62.0"] {
        assert_eq!(AmmlVersion::parse(bad), None, "{bad}");
    }
}

#[test]
fn displays_as_x_y_z_and_orders_by_part() {
    assert_eq!(version(26, 62, 0).to_string(), "26.62.0");
    assert!(version(26, 62, 0) > version(26, 60, 9));
    assert!(version(27, 0, 0) > version(26, 99, 99));
}
