use amml_launcher_core::{LauncherError, RegistryRef};

#[test]
fn a_plain_http_or_unparsable_registry_is_refused() {
    assert!(matches!(RegistryRef::parse("http://x/"), Err(LauncherError::BadSettings(_))));
    assert!(matches!(RegistryRef::parse("not a url"), Err(LauncherError::BadSettings(_))));
    assert!(matches!(RegistryRef::parse("https://"), Err(LauncherError::BadSettings(_))));
    assert_eq!(RegistryRef::parse("https://x.org/reg").unwrap().base, "https://x.org/reg/");
}

#[test]
fn cache_keys_differ_per_registry_and_are_path_safe() {
    let first = RegistryRef::parse("https://x.org/reg/").unwrap().cache_key();
    let second = RegistryRef::parse("https://y.org/reg/").unwrap().cache_key();
    assert_ne!(first, second);
    for key in [first, second] {
        assert_eq!(key.len(), 16);
        assert!(key.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
    }
}
