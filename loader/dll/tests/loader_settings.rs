use amml::{LoaderSettings, SettingsError};

#[test]
fn grade_is_off_by_default_and_an_empty_file_keeps_it_off() {
    assert!(!LoaderSettings::default().grade);
    assert!(!LoaderSettings::parse("").unwrap().grade);
}

#[test]
fn grade_can_be_switched_on() {
    assert!(LoaderSettings::parse("grade = true\n").unwrap().grade);
}

#[test]
fn an_unknown_attribute_is_rejected() {
    let error = LoaderSettings::parse("grade = true\ngrdae = true\n").unwrap_err();

    assert!(matches!(error, SettingsError::UnknownAttribute(name) if name == "grdae"));
}

#[test]
fn a_wrong_type_and_blocks_are_rejected() {
    assert!(matches!(LoaderSettings::parse("grade = \"yes\"").unwrap_err(), SettingsError::WrongType(_)));
    assert!(matches!(LoaderSettings::parse("grade {}\n").unwrap_err(), SettingsError::Blocks));
}

#[test]
fn a_missing_file_gives_the_defaults() {
    let dir = tempfile::tempdir().unwrap();

    let settings = LoaderSettings::load(&dir.path().join("amml.hcl")).unwrap();

    assert_eq!(settings, LoaderSettings::default());
}

#[test]
fn a_file_with_an_unknown_attribute_is_rejected_when_loaded() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("amml.hcl");
    std::fs::write(&path, "verbose = true\n").unwrap();

    let error = LoaderSettings::load(&path).unwrap_err();

    assert!(
        matches!(&error, SettingsError::File { source, .. } if matches!(**source, SettingsError::UnknownAttribute(_)))
    );
    assert!(error.to_string().contains("amml.hcl"));
}
