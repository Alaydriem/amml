use crate::support::{
    Bodies, FakeCall, FakeDeployer, FakeExeCopier, FakeInventory, FakeProcesses, FakeServer, GameFolder, Packages,
    Route,
};
use amml_gdk::{
    AmmlPaths, CancelFlag, CatalogueReader, Channel, DiskSpace, DownloadCache, Downloader, GameVersion, GdkError,
    InstallJournal, InstallLock, InstallOutcome, InstallRequest, InstallService, JournalStore, StoreCheck, StoreState,
};
use std::os::windows::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::time::Duration;

type Service = InstallService<FakeInventory, FakeProcesses, FakeInventory, FakeDeployer, FakeExeCopier>;

const PACKAGE_BYTES: usize = 3000;

struct Rig {
    root: tempfile::TempDir,
    server: FakeServer,
    inventory: FakeInventory,
    processes: FakeProcesses,
    deployer: FakeDeployer,
    exe: FakeExeCopier,
    paths: AmmlPaths,
    service: Service,
}

impl Rig {
    // `registered` is the third version part of the package the Store has installed
    fn new(registered: &str, exe: FakeExeCopier) -> Rig {
        let root = tempfile::tempdir().unwrap();
        let paths = AmmlPaths::new(root.path().join("amml"));
        let listed = ["6005", "5203", "6100"];
        let routes: Vec<(String, Route)> = listed
            .iter()
            .enumerate()
            .map(|(index, packed)| {
                let mut route = Route::new(Bodies::make(PACKAGE_BYTES, index as u8));
                route.etag = Some(format!("\"{packed}\""));
                (format!("/{}.msixvc", Packages::full_name(packed)), route)
            })
            .collect();
        let server = FakeServer::start(routes.iter().map(|(path, route)| (path.as_str(), route.clone())).collect());
        let entries: Vec<String> = listed
            .iter()
            .map(|packed| {
                let key = GameVersion::from_package_version(&format!("1.26.{packed}.0")).unwrap().to_string();
                let url = server.url(&format!("/{}.msixvc", Packages::full_name(packed)));
                format!("\"{key}\": [\"{url}\"]")
            })
            .collect();
        let preview_url = format!("http://127.0.0.1:9/{}.msixvc", Packages::preview_full_name("7022"));
        let json =
            format!("{{\"release\": {{{}}}, \"preview\": {{\"1.26.70.22\": [\"{preview_url}\"]}}}}", entries.join(","));
        let catalogue = CatalogueReader::parse(&json).unwrap();

        let packages_dir = root.path().join("packages");
        let inventory = FakeInventory::new();
        inventory.set(Packages::gaming_services(root.path()));
        let processes = FakeProcesses::new();
        let deployer = FakeDeployer::new(inventory.clone(), packages_dir, paths.downloads());
        deployer.register(&Packages::full_name(registered));

        let check = StoreCheck::new(inventory.clone(), processes.clone(), DiskSpace::new());
        let service = InstallService::new(
            paths.clone(),
            catalogue,
            Downloader::new(FakeServer::agent(), paths.downloads(), Downloader::DEFAULT_ATTEMPTS, CancelFlag::new()),
            DownloadCache::new(paths.downloads()),
            check,
            inventory.clone(),
            deployer.clone(),
            exe.clone(),
            Duration::ZERO,
        );
        Rig { root, server, inventory, processes, deployer, exe, paths, service }
    }

    fn install(&self, version: &str) -> Result<InstallOutcome, GdkError> {
        let request = InstallRequest { version: GameVersion::parse(version).unwrap(), channel: Channel::Release };
        self.service.install(&request, &mut |_, _| {})
    }

    fn registered_name(&self) -> String {
        use amml_gdk::PackageInventory;
        self.inventory.registered(Packages::RELEASE).unwrap().unwrap().full_name
    }

    fn version_dir(&self, name: &str) -> PathBuf {
        self.paths.versions().join(name)
    }

    fn staging_folders(&self) -> usize {
        match std::fs::read_dir(self.paths.versions()) {
            Ok(read) => {
                read.filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().starts_with(".staging-")).count()
            }
            Err(_) => 0,
        }
    }

    fn downloads(&self) -> Vec<String> {
        let mut names: Vec<String> = match std::fs::read_dir(self.paths.downloads()) {
            Ok(read) => read.map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect(),
            Err(_) => Vec::new(),
        };
        names.sort();
        names
    }
}

fn add(packed: &str) -> FakeCall {
    FakeCall::Add(Packages::full_name(packed))
}

fn remove(packed: &str) -> FakeCall {
    FakeCall::Remove(Packages::full_name(packed))
}

#[test]
fn current_version_is_copied_out_without_registration_change() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));

    let outcome = rig.install("1.26.60.5").unwrap();

    assert_eq!(outcome.path_used, "copy-out");
    assert!(rig.deployer.calls().is_empty());
    assert_eq!(outcome.version.path, rig.version_dir("1.26.60.05"));
    assert_eq!(outcome.version.files, GameFolder::FILES);
    assert!(outcome.version.path.join("MicrosoftGame.Config").is_file());
    assert_eq!(rig.registered_name(), Packages::full_name("6005"));
    assert_eq!(rig.staging_folders(), 0);
    assert!(rig.server.requests().is_empty());
}

#[test]
fn older_version_runs_install_then_restore_in_order() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));

    let outcome = rig.install("1.26.52.3").unwrap();

    assert_eq!(outcome.path_used, "install-restore");
    assert_eq!(rig.deployer.calls(), vec![add("5203"), remove("5203"), add("6005")]);
    let current_file = format!("{}.msixvc", Packages::full_name("6005"));
    let at_first_add = &rig.deployer.downloads_at_add()[0];
    assert_eq!(at_first_add.get(&current_file), Some(&(PACKAGE_BYTES as u64)));
    assert_eq!(rig.registered_name(), Packages::full_name("6005"));
    assert!(!outcome.store_updated_meanwhile);
    assert_eq!(outcome.final_full_name, Packages::full_name("6005"));
    assert_eq!(outcome.version.path, rig.version_dir("1.26.52.03"));
    assert_eq!(outcome.version.files, GameFolder::FILES);
    assert!(!rig.paths.journal().exists());
    assert_eq!(rig.staging_folders(), 0);
}

#[test]
fn exe_copy_failure_still_restores_and_leaves_no_version() {
    let rig = Rig::new("6005", FakeExeCopier::failing("the cmdlet is missing"));

    let result = rig.install("1.26.52.3");

    assert!(matches!(result, Err(GdkError::ExeCopyFailed { .. })));
    let calls = rig.deployer.calls();
    assert_eq!(calls[calls.len() - 2..], [remove("5203"), add("6005")]);
    assert!(!rig.version_dir("1.26.52.03").exists());
    assert_eq!(rig.staging_folders(), 0);
    assert!(!rig.paths.journal().exists());
    assert_eq!(rig.registered_name(), Packages::full_name("6005"));
    assert!(!rig.exe.destinations()[0].exists());
}

#[test]
fn add_failure_restores() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    rig.deployer.fail_call(0);

    let result = rig.install("1.26.52.3");

    assert!(matches!(result, Err(GdkError::Deploy { .. })));
    assert_eq!(rig.deployer.calls(), vec![add("5203")]);
    assert_eq!(rig.registered_name(), Packages::full_name("6005"));
    assert!(!rig.paths.journal().exists());
}

#[test]
fn restore_failure_is_reported_with_package_path() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    rig.deployer.fail_call(2);
    rig.deployer.fail_call(3);

    let result = rig.install("1.26.52.3");

    match result {
        Err(GdkError::RestoreFailed { original, package, .. }) => {
            assert_eq!(original, Packages::full_name("6005"));
            assert_eq!(package, rig.paths.downloads().join(format!("{}.msixvc", Packages::full_name("6005"))));
            assert!(package.is_file());
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(rig.deployer.calls(), vec![add("5203"), remove("5203"), add("6005"), add("6005")]);
    assert!(rig.paths.journal().exists());
}

#[test]
fn a_failed_restore_is_finished_by_recover() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    rig.deployer.fail_call(2);
    rig.deployer.fail_call(3);
    assert!(matches!(rig.install("1.26.52.3"), Err(GdkError::RestoreFailed { .. })));

    let restored = rig.service.recover().unwrap();

    assert_eq!(restored, Some(Packages::full_name("6005")));
    assert_eq!(rig.registered_name(), Packages::full_name("6005"));
    assert!(!rig.paths.journal().exists());
}

#[test]
fn crash_after_add_is_recovered_on_next_run() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    let original_package = rig.paths.downloads().join(format!("{}.msixvc", Packages::full_name("6005")));
    std::fs::create_dir_all(rig.paths.downloads()).unwrap();
    std::fs::write(&original_package, b"package").unwrap();
    rig.deployer.register(&Packages::full_name("5203"));
    let staging = rig.paths.versions().join(".staging-crashed");
    std::fs::create_dir_all(&staging).unwrap();
    std::fs::write(staging.join("partial.bin"), b"x").unwrap();
    JournalStore::new(rig.paths.journal())
        .write(&InstallJournal {
            original_full_name: Packages::full_name("6005"),
            original_package,
            wanted_full_name: Packages::full_name("5203"),
            staging: Some(staging.clone()),
            channel: Channel::Release,
        })
        .unwrap();

    let restored = rig.service.recover().unwrap();

    assert_eq!(restored, Some(Packages::full_name("6005")));
    assert_eq!(rig.deployer.calls(), vec![remove("5203"), add("6005")]);
    assert!(!staging.exists());
    assert!(!rig.paths.journal().exists());
    assert_eq!(rig.registered_name(), Packages::full_name("6005"));
}

#[test]
fn recover_without_a_journal_does_nothing() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    assert_eq!(rig.service.recover().unwrap(), None);
    assert!(rig.deployer.calls().is_empty());
}

#[test]
fn install_recovers_a_stopped_install_before_it_starts() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    let original_package = rig.paths.downloads().join(format!("{}.msixvc", Packages::full_name("6005")));
    std::fs::create_dir_all(rig.paths.downloads()).unwrap();
    std::fs::write(&original_package, b"package").unwrap();
    rig.deployer.register(&Packages::full_name("5203"));
    JournalStore::new(rig.paths.journal())
        .write(&InstallJournal {
            original_full_name: Packages::full_name("6005"),
            original_package,
            wanted_full_name: Packages::full_name("5203"),
            staging: None,
            channel: Channel::Release,
        })
        .unwrap();

    let outcome = rig.install("1.26.60.5").unwrap();

    assert_eq!(rig.deployer.calls(), vec![remove("5203"), add("6005")]);
    assert_eq!(outcome.path_used, "copy-out");
    assert!(!rig.paths.journal().exists());
}

#[test]
fn store_update_during_install_is_accepted() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    rig.deployer.register_after_remove(&Packages::full_name("6100"));

    let outcome = rig.install("1.26.52.3").unwrap();

    assert_eq!(rig.deployer.calls(), vec![add("5203"), remove("5203")]);
    assert!(outcome.store_updated_meanwhile);
    assert_eq!(outcome.final_full_name, Packages::full_name("6100"));
    assert_eq!(rig.registered_name(), Packages::full_name("6100"));
    assert!(!rig.paths.journal().exists());
}

#[test]
fn current_not_in_catalogue_refuses_before_any_change() {
    let rig = Rig::new("7000", FakeExeCopier::new(8192));

    let result = rig.install("1.26.52.3");

    assert!(matches!(result, Err(GdkError::CurrentNotInCatalogue(name)) if name == Packages::full_name("7000")));
    assert!(rig.server.requests().is_empty());
    assert!(rig.deployer.calls().is_empty());
    assert!(!rig.paths.journal().exists());
}

#[test]
fn current_not_in_catalogue_still_allows_copy_out() {
    let rig = Rig::new("7000", FakeExeCopier::new(8192));
    let outcome = rig.install("1.26.70.0").unwrap();
    assert_eq!(outcome.path_used, "copy-out");
    assert!(rig.deployer.calls().is_empty());
}

#[test]
fn not_ready_refuses_before_download() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    let package = Packages::bedrock("6005", &rig.root.path().join("packages"));
    rig.processes.start(42, package.install_location.join("Minecraft.Windows.exe"));

    let result = rig.install("1.26.52.3");

    assert!(
        matches!(&result, Err(GdkError::NotReady(state)) if **state == StoreState::MinecraftRunning { pids: vec![42] })
    );
    assert!(rig.server.requests().iter().all(|request| request.method == "HEAD"));
    assert!(rig.deployer.calls().is_empty());
    assert!(!rig.paths.journal().exists());
}

#[test]
fn a_loose_instance_running_does_not_block_an_install() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    rig.processes.start(7, rig.paths.instances().join("x").join("Minecraft.Windows.exe"));
    assert!(rig.install("1.26.52.3").is_ok());
}

#[test]
fn unknown_version_is_refused() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    assert!(matches!(rig.install("1.20.0.1"), Err(GdkError::UnknownVersion(_))));
    assert!(rig.deployer.calls().is_empty());
}

#[test]
fn an_installed_version_is_not_installed_again() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    rig.install("1.26.52.3").unwrap();
    let calls = rig.deployer.calls().len();

    let result = rig.install("1.26.52.3");

    assert!(matches!(result, Err(GdkError::VersionExists(name)) if name == "1.26.52.03"));
    assert_eq!(rig.deployer.calls().len(), calls);
}

#[test]
fn second_install_while_locked_is_busy() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    let _held = InstallLock::acquire(&rig.paths.lock()).unwrap();
    assert!(matches!(rig.install("1.26.52.3"), Err(GdkError::Busy)));
    assert!(matches!(rig.service.recover(), Err(GdkError::Busy)));
}

#[test]
fn cache_keeps_only_current_package_after_success() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));

    rig.install("1.26.52.3").unwrap();

    assert_eq!(rig.downloads(), vec![format!("{}.msixvc", Packages::full_name("6005"))]);
}

fn original_package_file(rig: &Rig) -> PathBuf {
    let path = rig.paths.downloads().join(format!("{}.msixvc", Packages::full_name("6005")));
    std::fs::create_dir_all(rig.paths.downloads()).unwrap();
    std::fs::write(&path, b"package").unwrap();
    path
}

fn write_journal(rig: &Rig, wanted: &str, staging: Option<PathBuf>) {
    JournalStore::new(rig.paths.journal())
        .write(&InstallJournal {
            original_full_name: Packages::full_name("6005"),
            original_package: original_package_file(rig),
            wanted_full_name: Packages::full_name(wanted),
            staging,
            channel: Channel::Release,
        })
        .unwrap();
}

#[test]
fn a_failed_add_that_took_the_original_files_is_restored() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    rig.deployer.fail_call_and_damage(0);

    let result = rig.install("1.26.52.3");

    assert!(matches!(result, Err(GdkError::Deploy { .. })));
    assert_eq!(rig.deployer.calls(), vec![add("5203"), add("6005")]);
    let package = rig.inventory.current().unwrap();
    assert!(package.install_location.join("Minecraft.Windows.exe").is_file());
    assert!(!rig.paths.journal().exists());
}

#[test]
fn a_wanted_package_with_another_full_name_is_still_removed() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    let real = "Microsoft.MinecraftUWP_1.26.5203.0_neutral__8wekyb3d8bbwe";
    rig.deployer.real_name(&Packages::full_name("5203"), real);

    let outcome = rig.install("1.26.52.3").unwrap();

    assert_eq!(rig.deployer.calls(), vec![add("5203"), FakeCall::Remove(real.to_string()), add("6005")]);
    assert!(!outcome.store_updated_meanwhile);
    assert_eq!(rig.registered_name(), Packages::full_name("6005"));
    assert!(!rig.paths.journal().exists());
}

#[test]
fn inventory_errors_while_restoring_do_not_stop_the_restore() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    write_journal(&rig, "5203", None);
    rig.deployer.register(&Packages::full_name("5203"));
    rig.inventory.fail_next_reads(2);

    let restored = rig.service.recover().unwrap();

    assert_eq!(restored, Some(Packages::full_name("6005")));
    assert_eq!(rig.deployer.calls(), vec![add("6005")]);
}

#[test]
fn inventory_errors_that_last_are_a_restore_failure_with_the_fix() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    write_journal(&rig, "5203", None);
    rig.deployer.register(&Packages::full_name("5203"));
    rig.inventory.fail_next_reads(4);

    let result = rig.service.recover();

    assert!(matches!(result, Err(GdkError::RestoreFailed { .. })), "{result:?}");
    assert_eq!(rig.deployer.calls(), vec![add("6005"), add("6005")]);
    assert!(rig.paths.journal().exists());
}

#[test]
fn recover_restores_when_the_staging_folder_cannot_be_deleted() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    let staging = rig.paths.versions().join(".staging-held");
    std::fs::create_dir_all(&staging).unwrap();
    let held_path = staging.join("held.bin");
    std::fs::write(&held_path, b"x").unwrap();
    let _held = std::fs::OpenOptions::new().read(true).share_mode(0).open(&held_path).unwrap();
    write_journal(&rig, "5203", Some(staging.clone()));
    rig.deployer.register(&Packages::full_name("5203"));

    let restored = rig.service.recover().unwrap();

    assert_eq!(restored, Some(Packages::full_name("6005")));
    assert!(!rig.paths.journal().exists());
    assert!(staging.exists());
}

#[test]
fn a_journal_cannot_make_recover_delete_a_folder_outside_the_version_store() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    let outside = rig.root.path().join("precious");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("keep.txt"), b"keep").unwrap();
    let wrong_place = rig.root.path().join(".staging-elsewhere");
    std::fs::create_dir_all(&wrong_place).unwrap();
    for staging in [outside.clone(), wrong_place.clone()] {
        write_journal(&rig, "5203", Some(staging));
        rig.deployer.register(&Packages::full_name("5203"));
        rig.service.recover().unwrap();
    }
    assert!(outside.join("keep.txt").is_file());
    assert!(wrong_place.is_dir());
}

#[test]
fn an_empty_journal_is_reported_as_unreadable() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    std::fs::create_dir_all(rig.paths.journal().parent().unwrap()).unwrap();
    std::fs::write(rig.paths.journal(), b"").unwrap();

    assert!(matches!(rig.service.recover(), Err(GdkError::BadJournal(_))));
    assert!(matches!(rig.install("1.26.52.3"), Err(GdkError::BadJournal(_))));
    assert!(rig.deployer.calls().is_empty());
}

#[test]
fn the_service_tells_the_deployer_never_to_remove_the_original() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));

    rig.install("1.26.52.3").unwrap();

    let protected = rig.deployer.protected_at_remove();
    assert_eq!(protected.len(), 1);
    assert!(protected[0].contains(&Packages::full_name("6005")));
    assert!(!protected[0].contains(&Packages::full_name("5203")));
}

#[test]
fn a_journal_that_names_the_original_as_wanted_removes_nothing() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    write_journal(&rig, "6005", None);

    let restored = rig.service.recover().unwrap();

    assert_eq!(restored, Some(Packages::full_name("6005")));
    assert!(rig.deployer.calls().is_empty());
    assert_eq!(rig.registered_name(), Packages::full_name("6005"));
}

#[test]
fn the_cached_package_of_the_other_channel_is_kept() {
    let rig = Rig::new("6005", FakeExeCopier::new(8192));
    rig.inventory
        .set(Packages::from_full_name(&Packages::preview_full_name("7022"), &rig.root.path().join("packages")));
    std::fs::create_dir_all(rig.paths.downloads()).unwrap();
    let preview_file = format!("{}.msixvc", Packages::preview_full_name("7022"));
    std::fs::write(rig.paths.downloads().join(&preview_file), b"preview package").unwrap();
    std::fs::write(rig.paths.downloads().join("old.msixvc"), b"old").unwrap();

    rig.install("1.26.52.3").unwrap();

    let mut expected = vec![format!("{}.msixvc", Packages::full_name("6005")), preview_file];
    expected.sort();
    assert_eq!(rig.downloads(), expected);
}
