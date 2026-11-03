use crate::support::Roots;

#[test]
fn paths_follow_the_documented_layout() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    assert_eq!(paths.settings(), root.path().join("launcher").join("settings.hcl"));
    assert_eq!(paths.registry_cache(), root.path().join("launcher").join("cache").join("registries"));
    assert_eq!(paths.default_data_dir(), root.path().join("data"));
    assert_eq!(paths.instance("a"), root.path().join("instances").join("a"));
}

#[test]
fn a_checked_instance_name_is_a_plain_name() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    assert_eq!(paths.checked_instance("one").unwrap(), paths.instance("one"));
    for bad in ["..", "..\\x", "a/b", ".hidden", ".staging-1", "", "CON"] {
        assert!(paths.checked_instance(bad).is_err(), "{bad}");
    }
}
