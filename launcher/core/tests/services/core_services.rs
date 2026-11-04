use crate::support::Roots;
use amml_gdk::CancelFlag;
use amml_launcher_core::{CoreServices, LauncherError};

#[test]
fn settings_name_the_official_override_first_when_one_is_given() -> Result<(), LauncherError> {
    let root = tempfile::tempdir().map_err(LauncherError::from)?;
    let services =
        CoreServices::new(Roots::paths(root.path()), Some("https://127.0.0.1:1/".to_string()), CancelFlag::new());
    let settings = services.settings()?;
    assert!(settings.registries[0].is_official());
    assert_eq!(settings.registries[0].base, "https://127.0.0.1:1/");
    Ok(())
}

#[test]
fn settings_keep_the_real_official_registry_without_an_override() -> Result<(), LauncherError> {
    let root = tempfile::tempdir().map_err(LauncherError::from)?;
    let services = CoreServices::new(Roots::paths(root.path()), None, CancelFlag::new());
    let settings = services.settings()?;
    assert_eq!(settings.registries[0].base, amml_launcher_core::LauncherSettings::OFFICIAL_REGISTRY);
    Ok(())
}
