use std::path::{Path, PathBuf};

use amml::{InstanceSettings, SettingsError};

#[test]
fn missing_file_is_no_redirect() {
    let dir = tempfile::tempdir().unwrap();

    let settings = InstanceSettings::load(&dir.path().join("instance.hcl"), Path::new(r"C:\g")).unwrap();

    assert_eq!(settings.data_dir, None);
}

#[test]
fn absolute_data_dir_is_kept() {
    let text = r#"data_dir = "C:\\amml\\data""#;

    let (settings, unknown) = InstanceSettings::parse_with_unknown(text, Path::new(r"C:\g")).unwrap();

    assert_eq!(settings.data_dir, Some(PathBuf::from(r"C:\amml\data")));
    assert!(unknown.is_empty());
}

#[test]
fn relative_data_dir_resolves_against_game() {
    let (settings, _) = InstanceSettings::parse_with_unknown(r#"data_dir = ".""#, Path::new(r"C:\g")).unwrap();

    assert_eq!(settings.data_dir, Some(PathBuf::from(r"C:\g\.")));
}

#[test]
fn unicode_path_survives() {
    let text = r#"data_dir = "C:\\Users\\Zoë\\data""#;

    let (settings, _) = InstanceSettings::parse_with_unknown(text, Path::new(r"C:\g")).unwrap();

    assert_eq!(settings.data_dir, Some(PathBuf::from(r"C:\Users\Zoë\data")));
}

#[test]
fn unknown_attribute_is_reported_not_fatal() {
    let text = "data_dir = \"C:\\\\d\"\ntheme = \"x\"\n";

    let (settings, unknown) = InstanceSettings::parse_with_unknown(text, Path::new(r"C:\g")).unwrap();

    assert_eq!(settings.data_dir, Some(PathBuf::from(r"C:\d")));
    assert_eq!(unknown, ["theme"]);
}

#[test]
fn wrong_type_is_error() {
    let error = InstanceSettings::parse_with_unknown("data_dir = 3", Path::new(r"C:\g")).unwrap_err();

    assert!(matches!(error, SettingsError::WrongType(name) if name == "data_dir"));
}

#[test]
fn a_file_on_disk_is_read_and_its_errors_name_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("instance.hcl");
    std::fs::write(&path, "data_dir = 3").unwrap();

    let error = InstanceSettings::load(&path, Path::new(r"C:\g")).unwrap_err();

    assert!(error.to_string().contains("instance.hcl"), "{error}");
}
