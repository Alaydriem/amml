use std::path::Path;

use amml::FolderRedirect;
use tempfile::tempdir;

use crate::support::FolderProbe;

const E_FAIL: i32 = 0x8000_4005_u32 as i32;

struct Units;

impl Units {
    fn text(units: &[u16]) -> String {
        let end = units.iter().position(|unit| *unit == 0).unwrap_or(units.len());
        String::from_utf16_lossy(&units[..end])
    }
}

struct Redirect;

impl Redirect {
    fn install(probe: &FolderProbe, dir: &Path) -> Result<FolderRedirect, amml::HookError> {
        // SAFETY: the probe module stays mapped for the life of the test process.
        unsafe { FolderRedirect::install(probe.base(), dir) }
    }

    fn on(probe: &FolderProbe, dir: &Path) -> FolderRedirect {
        Self::install(probe, dir).unwrap()
    }
}

#[test]
fn known_folder_roaming_is_redirected() {
    let probe = FolderProbe::load();
    let dir = tempdir().unwrap();
    let real = probe.known_folder_roaming();
    let _redirect = Redirect::on(&probe, dir.path());

    let seen = probe.known_folder_roaming();

    assert_eq!(seen, dir.path().to_string_lossy());
    assert_ne!(seen, real);
}

#[test]
fn known_folder_other_ids_pass_through() {
    let probe = FolderProbe::load();
    let dir = tempdir().unwrap();
    let real = probe.known_folder_local();
    let _redirect = Redirect::on(&probe, dir.path());

    assert_eq!(probe.known_folder_local(), real);
}

#[test]
fn folder_path_appdata_is_redirected_and_other_csidl_passes() {
    let probe = FolderProbe::load();
    let dir = tempdir().unwrap();
    let real_documents = probe.folder_path_personal();
    let _redirect = Redirect::on(&probe, dir.path());

    let (status, appdata) = probe.folder_path_appdata();

    assert_eq!(status, 0);
    assert_eq!(appdata, dir.path().to_string_lossy());
    assert_eq!(probe.folder_path_personal(), real_documents);
}

#[test]
fn folder_path_too_long_fails_without_writing() {
    let probe = FolderProbe::load();
    let dir = tempdir().unwrap();
    let long = dir.path().join("a".repeat(100)).join("b".repeat(100)).join("c".repeat(100));
    let _redirect = Redirect::on(&probe, &long);

    let (status, buffer) = probe.folder_path_appdata_filled(0x5A5A);

    assert_eq!(status, E_FAIL);
    assert!(buffer.iter().all(|unit| *unit == 0x5A5A));
    assert!(long.is_dir());
}

#[test]
fn env_appdata_contract() {
    let probe = FolderProbe::load();
    let dir = tempdir().unwrap();
    let real_path = probe.env("PATH", 32768, 0);
    let _redirect = Redirect::on(&probe, dir.path());
    let wanted = dir.path().to_string_lossy().into_owned();

    let (length, buffer) = probe.env("APPDATA", 32768, 0);
    assert_eq!(length as usize, wanted.encode_utf16().count());
    assert_eq!(Units::text(&buffer), wanted);

    let (lowercase_length, lowercase) = probe.env("appdata", 32768, 0);
    assert_eq!(lowercase_length, length);
    assert_eq!(Units::text(&lowercase), wanted);

    let (required, small) = probe.env("APPDATA", 4, 0x5A5A);
    assert_eq!(required, length + 1);
    assert!(small.iter().all(|unit| *unit == 0x5A5A));

    let (path_length, path) = probe.env("PATH", 32768, 0);
    assert_eq!(path_length, real_path.0);
    assert_eq!(Units::text(&path), Units::text(&real_path.1));

    let (longer_name, _) = probe.env("APPDATAX", 32768, 0);
    assert_eq!(longer_name, 0);
}

#[test]
fn missing_data_dir_is_created() {
    let probe = FolderProbe::load();
    let dir = tempdir().unwrap();
    let missing = dir.path().join("not").join("there").join("Zoë");
    assert!(!missing.exists());

    let _redirect = Redirect::on(&probe, &missing);

    assert!(missing.is_dir());
    assert_eq!(probe.known_folder_roaming(), missing.to_string_lossy());
}

#[test]
fn a_second_redirect_in_one_process_is_refused() {
    let probe = FolderProbe::load();
    let dir = tempdir().unwrap();
    let _first = Redirect::on(&probe, dir.path());
    let other = tempdir().unwrap();

    let second = Redirect::install(&probe, other.path());

    assert!(second.is_err());
    assert_eq!(probe.known_folder_roaming(), dir.path().to_string_lossy());
}

#[test]
fn restore_gives_the_real_folders_back() {
    let probe = FolderProbe::load();
    let dir = tempdir().unwrap();
    let real = probe.known_folder_roaming();
    let redirect = Redirect::on(&probe, dir.path());
    assert!(!redirect.slots().is_empty());

    redirect.restore();

    assert_eq!(probe.known_folder_roaming(), real);
}

#[test]
fn env_appdata_size_query_with_a_null_buffer_returns_the_required_size() {
    let probe = FolderProbe::load();
    let dir = tempdir().unwrap();
    let _redirect = Redirect::on(&probe, dir.path());
    let wanted = dir.path().to_string_lossy().encode_utf16().count();

    assert_eq!(probe.env_size() as usize, wanted + 1);
}

#[test]
fn a_module_that_imports_none_of_the_functions_is_refused() {
    let dir = tempdir().unwrap();
    let ntdll = Path::new(&std::env::var("SystemRoot").unwrap()).join("System32").join("ntdll.dll");
    let module = amml::Library::open(&ntdll).unwrap();

    // SAFETY: ntdll stays mapped for the life of the process.
    let result = unsafe { FolderRedirect::install(module.base(), dir.path()) };

    assert!(matches!(result, Err(amml::HookError::ImportTable(_))));
}
