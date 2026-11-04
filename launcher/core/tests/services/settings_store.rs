use crate::support::Roots;
use amml_launcher_core::{LauncherError, LauncherSettings, RegistryRef, SettingsStore};

#[test]
fn a_missing_file_gives_the_official_registry_only() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let settings = SettingsStore::new(paths.clone()).load().unwrap();
    assert_eq!(settings.registries.len(), 1);
    assert_eq!(settings.registries[0].base, LauncherSettings::OFFICIAL_REGISTRY);
    assert!(!settings.preview);
    assert_eq!(settings.data_dir(&paths), root.path().join("data"));
}

#[test]
fn the_official_registry_is_always_first_and_never_removed() {
    let root = tempfile::tempdir().unwrap();
    let store = SettingsStore::new(Roots::paths(root.path()));
    let official = LauncherSettings::official();
    let other = RegistryRef::parse("https://example.org/reg/").unwrap();

    store.save(&LauncherSettings::new(vec![other.clone()], false, None)).unwrap();
    assert_eq!(store.load().unwrap().registries, vec![official.clone(), other.clone()]);

    store.save(&LauncherSettings::new(vec![other.clone(), official.clone()], true, None)).unwrap();
    let loaded = store.load().unwrap();
    assert_eq!(loaded.registries, vec![official.clone(), other.clone()]);
    assert!(loaded.preview);

    store.save(&LauncherSettings::new(vec![other.clone(), other.clone(), official.clone()], false, None)).unwrap();
    assert_eq!(store.load().unwrap().registries, vec![official, other]);
}

#[test]
fn a_data_dir_with_backslashes_and_quotes_is_read_back() {
    let root = tempfile::tempdir().unwrap();
    let store = SettingsStore::new(Roots::paths(root.path()));
    let dir = std::path::PathBuf::from("D:\\Minecraft \"x\"\\o'brien\\data");
    store.save(&LauncherSettings::new(Vec::new(), false, Some(dir.clone()))).unwrap();
    assert_eq!(store.load().unwrap().data_dir, Some(dir));
}

#[test]
fn an_unknown_key_or_wrong_type_is_a_named_error() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    std::fs::create_dir_all(paths.settings().parent().unwrap()).unwrap();
    let store = SettingsStore::new(paths.clone());

    std::fs::write(paths.settings(), "previw = true\n").unwrap();
    match store.load() {
        Err(LauncherError::BadSettings(text)) => assert!(text.contains("previw"), "{text}"),
        other => panic!("{other:?}"),
    }

    std::fs::write(paths.settings(), "preview = \"yes\"\n").unwrap();
    match store.load() {
        Err(LauncherError::BadSettings(text)) => assert!(text.contains("preview"), "{text}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_settings_file_that_still_holds_the_retired_download_limit_is_read_and_the_key_is_dropped_on_save() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    std::fs::create_dir_all(paths.settings().parent().unwrap()).unwrap();
    std::fs::write(
        paths.settings(),
        "registries = [\"https://example.org/reg/\"]\npreview = true\ndownload_limit_bytes = 10000000000\n",
    )
    .unwrap();
    let store = SettingsStore::new(paths.clone());

    let settings = store.load().unwrap();
    assert!(settings.preview);
    assert_eq!(settings.registries.len(), 2);
    store.save(&settings).unwrap();

    let text = std::fs::read_to_string(paths.settings()).unwrap();
    assert!(!text.contains("download_limit"), "{text}");
    assert_eq!(store.load().unwrap(), settings);
}

#[test]
fn a_retired_download_limit_of_any_type_does_not_fail_the_read() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    std::fs::create_dir_all(paths.settings().parent().unwrap()).unwrap();
    std::fs::write(paths.settings(), "download_limit_bytes = \"lots\"\n").unwrap();

    assert!(SettingsStore::new(paths).load().is_ok());
}
