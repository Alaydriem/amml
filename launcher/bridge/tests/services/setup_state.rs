use crate::support::Samples;
use amml_launcher_bridge::SetupState;
use amml_launcher_core::{LauncherSettings, SettingsStore};

#[test]
fn a_pc_without_amml_needs_the_first_run() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());

    assert!(!SetupState::installed(&paths));
    assert!(SetupState::view(&paths).needed);
}

#[test]
fn amml_is_installed_when_the_settings_file_exists() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    SettingsStore::new(paths.clone()).save(&LauncherSettings::new(Vec::new(), false, None)).unwrap();

    assert!(SetupState::installed(&paths));
    assert!(!SetupState::view(&paths).needed);
}

#[test]
fn amml_is_installed_when_an_instance_folder_exists() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    std::fs::create_dir_all(paths.gdk().instances().join("1.26.60.05")).unwrap();

    assert!(SetupState::installed(&paths));
    assert!(!SetupState::view(&paths).needed);
}

#[test]
fn amml_is_installed_when_a_version_folder_exists() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    std::fs::create_dir_all(paths.gdk().versions().join("1.26.60.05")).unwrap();

    assert!(SetupState::installed(&paths));
}

#[test]
fn the_logs_folder_the_app_makes_at_start_does_not_count_as_amml() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    std::fs::create_dir_all(dir.path().join("launcher").join("logs")).unwrap();
    std::fs::create_dir_all(paths.gdk().instances()).unwrap();
    std::fs::create_dir_all(paths.gdk().versions()).unwrap();

    assert!(!SetupState::installed(&paths));
    assert!(SetupState::view(&paths).needed);
}

#[test]
fn hidden_folders_and_files_in_instances_and_versions_do_not_count() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    std::fs::create_dir_all(paths.gdk().instances().join(".staging-abc")).unwrap();
    std::fs::create_dir_all(paths.gdk().versions().join(".staging-def")).unwrap();
    std::fs::write(paths.gdk().instances().join("notes.txt"), b"x").unwrap();
    std::fs::write(paths.gdk().versions().join("readme"), b"x").unwrap();

    assert!(!SetupState::installed(&paths));
}
