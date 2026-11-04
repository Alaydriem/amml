use crate::support::{FakeServer, Route};
use amml_gdk::{CancelFlag, Downloader, GdkError};
use amml_launcher_core::{AssetFetcher, CoreProgress, GameInput, GameInputPackage, LauncherError, UreqHttpFetch};

fn game_input(dir: &std::path::Path, package: GameInputPackage, cancel: CancelFlag) -> GameInput {
    let downloader = Downloader::new(UreqHttpFetch::agent(), dir.to_path_buf(), 1, cancel.clone());
    GameInput::new(AssetFetcher::new(downloader, dir.to_path_buf()), cancel, package)
}

#[test]
fn a_service_state_line_means_installed() {
    assert!(GameInput::is_installed_in(
        "SERVICE_NAME: GameInputSvc\n        TYPE : 10 WIN32_OWN_PROCESS\n        STATE : 4 RUNNING\n"
    ));
    assert!(!GameInput::is_installed_in("[SC] EnumQueryServicesStatus:OpenService FAILED 1060:\n"));
}

#[test]
fn a_package_with_another_hash_is_refused_and_nothing_runs() {
    let server = FakeServer::start(vec![("/gameinput.nupkg", Route::new(vec![7u8; 4000]))]);
    let dir = tempfile::tempdir().unwrap();
    let mut package = GameInputPackage::official();
    package.url = server.url("/gameinput.nupkg");
    let input = game_input(dir.path(), package, CancelFlag::new());

    let result = input.install(&mut |_: CoreProgress| {});

    assert!(matches!(result, Err(LauncherError::ChecksumMismatch { .. })));
    assert!(std::fs::read_dir(dir.path()).unwrap().next().is_none());
}

#[test]
fn a_cancelled_flag_stops_the_install_before_any_request() {
    let server = FakeServer::start(vec![("/gameinput.nupkg", Route::new(vec![7u8; 4000]))]);
    let dir = tempfile::tempdir().unwrap();
    let mut package = GameInputPackage::official();
    package.url = server.url("/gameinput.nupkg");
    let cancel = CancelFlag::new();
    cancel.cancel();
    let input = game_input(dir.path(), package, cancel);

    let result = input.install(&mut |_: CoreProgress| {});

    assert!(matches!(result, Err(LauncherError::Gdk(GdkError::Cancelled))));
    assert!(server.requests().is_empty());
}

#[test]
fn an_msi_with_another_hash_is_refused_and_nothing_is_left_behind() {
    let work = tempfile::tempdir().unwrap();
    let zip = crate::support::Zips::raw(
        &work.path().join("package.nupkg"),
        &[("redist/GameInputRedist.msi", b"not the real installer".as_slice())],
    );
    let bytes = std::fs::read(&zip).unwrap();
    let server = FakeServer::start(vec![("/gameinput.nupkg", Route::new(bytes.clone()))]);
    let dir = tempfile::tempdir().unwrap();
    let mut package = GameInputPackage::official();
    package.url = server.url("/gameinput.nupkg");
    // the package hash is right, so only the hash of the MSI inside it can refuse the install
    package.sha256 = amml_registry::HashText::sha256(&bytes);
    let input = game_input(dir.path(), package, CancelFlag::new());

    let result = input.install(&mut |_: CoreProgress| {});

    assert!(
        matches!(result, Err(LauncherError::ChecksumMismatch { file, .. }) if file == "redist/GameInputRedist.msi")
    );
    let left: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert!(left.is_empty(), "{left:?}");
}

#[test]
fn an_installer_with_the_pinned_hash_is_held_against_writes_and_one_with_another_hash_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("setup.msi");
    std::fs::write(&path, b"abc").unwrap();
    let mut package = GameInputPackage::official();
    package.msi_sha256 = "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD".to_string();
    let input = game_input(dir.path(), package.clone(), CancelFlag::new());

    let hold = input.hold_checked(&path).unwrap();

    assert!(std::fs::OpenOptions::new().write(true).open(&path).is_err());
    assert!(std::fs::remove_file(&path).is_err());
    drop(hold);
    assert!(std::fs::OpenOptions::new().write(true).open(&path).is_ok());

    std::fs::write(&path, b"swapped").unwrap();
    let refused = input.hold_checked(&path);
    assert!(matches!(refused, Err(LauncherError::ChecksumMismatch { .. })));
    assert!(std::fs::OpenOptions::new().write(true).open(&path).is_ok());
}
