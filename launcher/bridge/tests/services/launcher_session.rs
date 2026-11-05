use crate::support::{RecordingSink, Samples, StallingHost};
use amml_gdk::{CancelFlag, GdkError, InstallJournal, JournalStore};
use amml_launcher_bridge::{
    BridgeError, CatalogSortView, FaultKindView, InstallSteps, JobError, JobKind, JobStateView, LauncherSession,
    RegistryStatusView,
};
use amml_launcher_core::{CoreServices, LauncherError, RegistryRef, SettingsStore};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

// how long a test waits for a reader to reach its host
const READER_START_LIMIT: Duration = Duration::from_secs(5);

fn session(root: &Path) -> LauncherSession<RecordingSink> {
    let cancel = CancelFlag::new();
    let services = CoreServices::new(Samples::paths(root), None, cancel.clone());
    LauncherSession::new(services, RecordingSink::new(), cancel)
}

fn downloads(root: &Path) -> std::path::PathBuf {
    let dir = Samples::paths(root).gdk().downloads();
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// starts a job that runs until the returned sender is used
fn blocking_job(session: &LauncherSession<RecordingSink>) -> mpsc::Sender<()> {
    let (release, gate) = mpsc::channel::<()>();
    session
        .runner()
        .start(JobKind::UpdateAmml { name: "play".to_string() }, InstallSteps::for_amml(), move |_| {
            let _ = gate.recv_timeout(Duration::from_secs(20));
            Ok(())
        })
        .unwrap();
    release
}

#[test]
fn a_second_action_while_a_job_runs_is_refused_with_busy_and_changes_nothing_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());
    let downloads = downloads(dir.path());
    std::fs::write(downloads.join("a.zip"), b"zip").unwrap();
    std::fs::write(downloads.join("b.zip.part"), b"part").unwrap();
    let settings_before = std::fs::read_to_string(Samples::paths(dir.path()).settings()).ok();
    let release = blocking_job(&session);

    let results = [
        session.set_mod_enabled("play", "@a/b", false).err(),
        session.remove_mod("play", "@a/b").err(),
        session.delete("play", "play").err(),
        session.mods("play").err(),
        session.catalog_search("play", "x").err(),
        session.catalog_sorted("play", "r", CatalogSortView::Size).err(),
        session.inspect_url("play", "https://x/a.zip").err(),
        session.launch("play").err(),
        session.download_remove("a.zip").err(),
        session.downloads_remove_unfinished().err(),
        session.remove_version("1.26.60.5").err(),
        session.set_preview(true).err(),
        session.add_registry("https://example.org/").err(),
        session.remove_registry("https://example.org/").err(),
    ];

    for (index, error) in results.iter().enumerate() {
        assert!(matches!(error, Some(BridgeError::Busy)), "call {index}: {error:?}");
    }
    assert!(downloads.join("a.zip").is_file());
    assert!(downloads.join("b.zip.part").is_file());
    assert_eq!(std::fs::read_to_string(Samples::paths(dir.path()).settings()).ok(), settings_before);
    release.send(()).unwrap();
    session.runner().wait();
}

#[test]
fn a_registry_refresh_runs_while_a_job_runs() {
    let dir = tempfile::tempdir().unwrap();
    let session = offline_session(dir.path());
    let release = blocking_job(&session);

    let refreshed = session.refresh_registries();

    assert!(refreshed.is_ok(), "{refreshed:?}");
    release.send(()).unwrap();
    session.runner().wait();
}

#[test]
fn a_refresh_with_no_network_sends_offline_views() {
    let dir = tempfile::tempdir().unwrap();
    let sink = RecordingSink::new();
    let cancel = CancelFlag::new();
    let services =
        CoreServices::new(Samples::paths(dir.path()), Some("https://127.0.0.1:1/".to_string()), cancel.clone());
    let session = LauncherSession::new(services, sink.clone(), cancel);

    let views = session.refresh_registries().unwrap();

    assert!(!views.is_empty());
    assert!(views.iter().all(|view| view.fetched.is_none() && view.status != RegistryStatusView::Ok), "{views:?}");
    assert_eq!(sink.registry_events(), vec![views.clone()]);
    assert_eq!(session.registry_views(), views);
}

#[test]
fn a_second_job_while_one_runs_is_refused_by_the_session() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());
    let release = blocking_job(&session);

    let refused = session.install_game_input();

    assert!(matches!(refused, Err(BridgeError::Busy)));
    release.send(()).unwrap();
    session.runner().wait();
}

#[test]
fn downloads_may_be_removed_once_no_job_runs() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());
    let downloads = downloads(dir.path());
    std::fs::write(downloads.join("a.zip"), b"zip").unwrap();
    std::fs::write(downloads.join("b.zip.part"), b"part").unwrap();
    let release = blocking_job(&session);
    assert!(session.download_remove("a.zip").is_err());
    release.send(()).unwrap();
    session.runner().wait();

    session.download_remove("a.zip").unwrap();
    assert_eq!(session.downloads_remove_unfinished().unwrap(), 1);

    assert!(!downloads.join("a.zip").exists());
    assert!(!downloads.join("b.zip.part").exists());
}

#[test]
fn the_package_a_cut_off_install_puts_back_is_never_removed_while_its_journal_exists() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());
    let paths = Samples::paths(dir.path());
    let downloads = downloads(dir.path());
    std::fs::write(downloads.join("original.msixvc"), b"the package a restore needs").unwrap();
    let journal = InstallJournal {
        original_full_name: "Microsoft.MinecraftUWP_1.0.0.0_x64__8wekyb3d8bbwe".to_string(),
        original_package: downloads.join("original.msixvc"),
        wanted_full_name: "Microsoft.MinecraftUWP_2.0.0.0_x64__8wekyb3d8bbwe".to_string(),
        staging: None,
        channel: amml_gdk::Channel::Release,
    };
    JournalStore::new(paths.gdk().journal()).write(&journal).unwrap();

    let refused = session.download_remove("original.msixvc");
    let shown = session.downloads().unwrap();

    assert!(matches!(refused, Err(BridgeError::Refused(_))));
    assert!(downloads.join("original.msixvc").is_file());
    let held = shown.files.iter().find(|file| file.id == "original.msixvc").unwrap();
    assert_eq!(held.state, amml_launcher_bridge::DownloadFileStateView::Kept);
}

#[test]
fn developer_mode_is_turned_on_only_after_an_install_failed_for_the_exe_copy() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());

    assert!(matches!(session.retry_with_developer_mode(), Err(BridgeError::Refused(_))));

    session
        .runner()
        .start(JobKind::UpdateAmml { name: "play".to_string() }, InstallSteps::for_amml(), |_| {
            Err(JobError::Launcher(LauncherError::UnknownMod("@a/b".to_string())))
        })
        .unwrap();
    session.runner().wait();
    let snapshot = session.job().unwrap();
    assert_eq!(snapshot.state, JobStateView::Failed);
    assert_eq!(snapshot.fault.unwrap().kind, FaultKindView::OperationFailed);
    assert!(matches!(session.retry_with_developer_mode(), Err(BridgeError::Refused(_))));

    let release = blocking_job(&session);
    assert!(matches!(session.retry_with_developer_mode(), Err(BridgeError::Busy)));
    release.send(()).unwrap();
    session.runner().wait();
}

#[test]
fn a_failed_exe_copy_ends_the_job_with_the_consent_needed_fault_that_unlocks_developer_mode() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());
    session
        .runner()
        .start(JobKind::UpdateAmml { name: "play".to_string() }, InstallSteps::for_amml(), |_| {
            Err(JobError::Gdk(GdkError::ExeCopyFailed { method: "plain", detail: "denied".to_string() }))
        })
        .unwrap();
    session.runner().wait();

    assert_eq!(session.job().unwrap().fault.unwrap().kind, FaultKindView::ConsentNeeded);
}

fn make_unmanaged_instance(root: &Path, name: &str) -> std::path::PathBuf {
    let folder = Samples::paths(root).gdk().instances().join(name);
    std::fs::create_dir_all(&folder).unwrap();
    let text = format!(
        "{{\"name\":\"{name}\",\"gameVersion\":\"1.26.60.05\",\"type\":\"release\",\"enableIsolation\":true,\"launchArgs\":\"\",\"envVars\":\"\"}}"
    );
    std::fs::write(folder.join("version.json"), text).unwrap();
    std::fs::write(folder.join("Minecraft.Windows.exe.bak"), b"x").unwrap();
    folder
}

fn run_setup(session: &LauncherSession<RecordingSink>, source: &str) -> amml_launcher_bridge::JobSnapshot {
    session.set_up_data(source).unwrap();
    session.runner().wait();
    session.job().unwrap()
}

#[test]
fn a_pc_with_amml_and_no_settings_gets_default_settings_and_no_instance_file_changes() {
    let dir = tempfile::tempdir().unwrap();
    let folder = make_unmanaged_instance(dir.path(), "1.26.60.05");
    let version_json = folder.join("version.json");
    let before = (std::fs::read(&version_json).unwrap(), std::fs::metadata(&version_json).unwrap().modified().unwrap());
    let session = session(dir.path());
    let paths = Samples::paths(dir.path());
    assert!(!paths.settings().exists());

    let setup = session.setup();

    assert!(!setup.needed);
    assert!(paths.settings().is_file());
    let after = (std::fs::read(&version_json).unwrap(), std::fs::metadata(&version_json).unwrap().modified().unwrap());
    assert_eq!(before, after);
    assert!(!folder.join("amml").join("instance.hcl").exists());
    let mut files: Vec<String> = std::fs::read_dir(&folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    files.sort();
    assert_eq!(files, vec!["Minecraft.Windows.exe.bak", "version.json"]);
    // the unmanaged instance is listed as it is
    let listed = session.instances().unwrap();
    assert_eq!(listed.len(), 1);
    assert!(!listed[0].managed);
}

#[test]
fn a_pc_without_amml_needs_the_first_run_and_gets_no_settings_from_the_check() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());

    let setup = session.setup();

    assert!(setup.needed);
    assert!(!Samples::paths(dir.path()).settings().exists());
}

#[test]
fn the_setup_data_job_copies_the_worlds_and_then_saves_the_settings() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    let source = paths.gdk().instances().join("old").join("Minecraft Bedrock");
    let world = source.join("Users").join("u").join("games").join("com.mojang").join("minecraftWorlds").join("w1");
    std::fs::create_dir_all(&world).unwrap();
    std::fs::write(world.join("level.dat"), b"level").unwrap();
    let session = session(dir.path());
    assert!(!paths.settings().exists());

    let snapshot = run_setup(&session, "old");

    assert_eq!(snapshot.state, JobStateView::Done, "{:?}", snapshot.fault);
    let copied =
        paths.default_data_dir().join("Minecraft Bedrock").join("Users").join("u").join("games").join("com.mojang");
    assert_eq!(std::fs::read(copied.join("minecraftWorlds").join("w1").join("level.dat")).unwrap(), b"level");
    assert!(paths.settings().is_file());
    assert!(world.join("level.dat").is_file());
    let ids: Vec<&str> = snapshot.steps.iter().map(|step| step.id.as_str()).collect();
    assert_eq!(ids, vec!["setup-check", "setup-measure", "setup-copy", "setup-settings"]);
    assert!(snapshot.steps.iter().all(|step| step.state == amml_launcher_bridge::JobStepStateView::Done));
    assert_eq!(snapshot.title_name, "old");
}

#[test]
fn a_failed_settings_save_removes_the_fresh_copy() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    let source = paths.gdk().instances().join("old").join("Minecraft Bedrock");
    std::fs::create_dir_all(source.join("minecraftWorlds").join("w1")).unwrap();
    std::fs::write(source.join("minecraftWorlds").join("w1").join("level.dat"), b"level").unwrap();
    // a folder where the settings temp file goes makes the save fail
    std::fs::create_dir_all(paths.settings().with_extension("hcl.tmp")).unwrap();
    let session = session(dir.path());

    let snapshot = run_setup(&session, "old");

    assert_eq!(snapshot.state, JobStateView::Failed);
    assert!(snapshot.fault.unwrap().restored);
    assert!(!paths.default_data_dir().join("Minecraft Bedrock").exists());
    assert!(source.join("minecraftWorlds").join("w1").join("level.dat").is_file());
}

#[test]
fn a_clean_start_saves_the_settings_and_copies_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    let session = session(dir.path());

    let snapshot = run_setup(&session, "scratch");

    assert_eq!(snapshot.state, JobStateView::Done);
    assert!(paths.settings().is_file());
    assert!(!paths.default_data_dir().join("Minecraft Bedrock").exists());
}

#[test]
fn a_not_empty_shared_data_folder_is_merged_and_settings_are_saved() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    let source = paths.gdk().instances().join("old").join("Minecraft Bedrock");
    std::fs::create_dir_all(source.join("minecraftWorlds").join("w1")).unwrap();
    std::fs::write(source.join("minecraftWorlds").join("w1").join("level.dat"), b"level").unwrap();
    let target = paths.default_data_dir().join("Minecraft Bedrock");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("keep.txt"), b"mine").unwrap();
    let session = session(dir.path());

    let snapshot = run_setup(&session, "old");

    assert_eq!(snapshot.state, JobStateView::Done);
    assert!(paths.settings().exists());
    assert_eq!(std::fs::read(target.join("keep.txt")).unwrap(), b"mine");
    assert!(target.join("minecraftWorlds").join("w1").join("level.dat").is_file());
}

// a session whose official registry cannot be reached, so a refresh answers at once and touches no real host
fn offline_session(root: &Path) -> LauncherSession<RecordingSink> {
    let cancel = CancelFlag::new();
    let services = CoreServices::new(Samples::paths(root), Some("https://127.0.0.1:1/".to_string()), cancel.clone());
    LauncherSession::new(services, RecordingSink::new(), cancel)
}

#[test]
fn a_refresh_after_a_registry_is_removed_sees_the_new_registry_list() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    let session = offline_session(dir.path());
    let store = SettingsStore::new(paths.clone());
    let mut settings = store.load().unwrap();
    settings.registries.push(RegistryRef::parse("https://127.0.0.1:2/").unwrap());
    store.save(&settings).unwrap();
    let before = session.refresh_registries().unwrap();
    assert_eq!(before.len(), 2);

    session.remove_registry("https://127.0.0.1:2/").unwrap();
    let after = session.refresh_registries().unwrap();

    assert_eq!(after.len(), 1);
    assert!(after[0].official);
    assert_eq!(store.load().unwrap().registries.len(), 1);
}

#[test]
fn a_launch_of_another_instance_is_not_refused_as_busy_while_a_job_runs_for_one_instance() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());
    let release = blocking_job(&session);

    let same = session.launch("play");
    let other = session.launch("other");

    assert!(matches!(same, Err(BridgeError::Busy)));
    assert!(!matches!(other, Err(BridgeError::Busy)), "{other:?}");
    release.send(()).unwrap();
    session.runner().wait();
}

#[test]
fn a_launch_of_any_instance_is_refused_as_busy_while_an_app_wide_job_runs() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());
    let (release, gate) = mpsc::channel::<()>();
    session
        .runner()
        .start(JobKind::InstallGameInput, InstallSteps::single("game-input"), move |_| {
            let _ = gate.recv_timeout(Duration::from_secs(20));
            Ok(())
        })
        .unwrap();

    let refused = session.launch("other");

    assert!(matches!(refused, Err(BridgeError::Busy)));
    release.send(()).unwrap();
    session.runner().wait();
}

#[test]
fn removing_a_registry_does_not_wait_for_a_reader_and_the_reader_does_not_keep_the_old_list() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    let official = StallingHost::start(Duration::from_millis(700));
    let extra = StallingHost::start(Duration::from_millis(700));
    let cancel = CancelFlag::new();
    let services = CoreServices::new(paths.clone(), Some(official.url()), cancel.clone());
    let session = std::sync::Arc::new(LauncherSession::new(services, RecordingSink::new(), cancel));
    let store = SettingsStore::new(paths);
    let mut settings = store.load().unwrap();
    settings.registries.push(RegistryRef::parse(&extra.url()).unwrap());
    store.save(&settings).unwrap();
    make_unmanaged_instance(dir.path(), "play");
    let reader = {
        let session = std::sync::Arc::clone(&session);
        std::thread::spawn(move || session.refresh_registries())
    };
    // a catalog search holds the manager from the moment it connects to the first host
    let manager_reader = {
        let session = std::sync::Arc::clone(&session);
        std::thread::spawn(move || session.catalog_search("play", "x"))
    };
    // the refresh reader reads the registries without the manager slot
    let deadline = std::time::Instant::now() + READER_START_LIMIT;
    while official.accepted() == 0 {
        assert!(std::time::Instant::now() < deadline, "the reader never reached the host");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!reader.is_finished());

    let began = std::time::Instant::now();
    session.remove_registry(&extra.url()).unwrap();
    let waited = began.elapsed();

    assert!(waited < Duration::from_millis(600), "{waited:?}");
    assert!(!reader.is_finished());
    reader.join().unwrap().unwrap();
    let _ = manager_reader.join();
    assert_eq!(session.refresh_registries().unwrap().len(), 1);
}

#[test]
fn a_clean_start_does_not_wait_for_a_reader_that_holds_the_manager() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    make_unmanaged_instance(dir.path(), "play");
    let official = StallingHost::start(Duration::from_secs(3));
    let cancel = CancelFlag::new();
    let services = CoreServices::new(paths, Some(official.url()), cancel.clone());
    let sink = RecordingSink::new();
    let session = std::sync::Arc::new(LauncherSession::new(services, sink.clone(), cancel));
    let reader = {
        let session = std::sync::Arc::clone(&session);
        std::thread::spawn(move || session.catalog_search("play", "x"))
    };
    // the reader holds the manager from the moment it connects to the host
    let deadline = std::time::Instant::now() + READER_START_LIMIT;
    while official.accepted() == 0 {
        assert!(std::time::Instant::now() < deadline, "the reader never reached the host");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!reader.is_finished());

    let began = std::time::Instant::now();
    let job = run_setup(&session, "scratch");
    let waited = began.elapsed();

    assert!(waited < Duration::from_secs(1), "{waited:?}");
    assert_eq!(job.state, amml_launcher_bridge::JobStateView::Done);
    assert!(!reader.is_finished());
    assert!(sink.snapshots().iter().all(|snapshot| snapshot.steps.iter().all(|step| step.id != "manager-wait")));
    let _ = reader.join();
}

#[test]
fn a_retry_after_consent_does_not_run_when_another_job_ran_during_the_prompt() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());
    session.update_amml("play").unwrap();
    session.runner().wait();
    session
        .runner()
        .start(JobKind::UpdateAmml { name: "play".to_string() }, InstallSteps::for_amml(), |_| {
            Err(JobError::Gdk(GdkError::ExeCopyFailed { method: "plain", detail: "denied".to_string() }))
        })
        .unwrap();
    session.runner().wait();
    assert_eq!(session.job().unwrap().fault.unwrap().kind, FaultKindView::ConsentNeeded);
    let before = session.job().unwrap().id;

    let answer = session.retry_after_consent(|| {
        session.runner().start(JobKind::InstallGameInput, InstallSteps::single("game-input"), |_| Ok(())).unwrap();
        session.runner().wait();
        Ok(true)
    });

    assert!(matches!(answer, Err(BridgeError::Busy)), "{answer:?}");
    assert_eq!(session.job().unwrap().id, before + 1);
    assert_eq!(session.job().unwrap().kind, "install-game-input");
}

#[test]
fn a_retry_after_consent_runs_the_job_that_failed_and_a_no_runs_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());
    session.update_amml("play").unwrap();
    session.runner().wait();
    session
        .runner()
        .start(JobKind::UpdateAmml { name: "play".to_string() }, InstallSteps::for_amml(), |_| {
            Err(JobError::Gdk(GdkError::ExeCopyFailed { method: "plain", detail: "denied".to_string() }))
        })
        .unwrap();
    session.runner().wait();
    let before = session.job().unwrap().id;

    let declined = session.retry_after_consent(|| Ok(false));
    let id = session.retry_after_consent(|| Ok(true)).unwrap();
    session.runner().wait();

    assert!(matches!(declined, Err(BridgeError::Unavailable(_))));
    assert_eq!(id, before + 1);
    assert_eq!(session.job().unwrap().kind, "update-amml");
}

#[test]
fn guarded_calls_answer_busy_while_a_launch_is_in_flight_and_work_after_it() {
    let dir = tempfile::tempdir().unwrap();
    let session = session(dir.path());
    let downloads = downloads(dir.path());
    std::fs::write(downloads.join("a.zip"), b"zip").unwrap();
    let ticket = session.runner().begin_launch("play").unwrap();

    let removed = session.download_remove("a.zip");
    let preview = session.set_preview(true);

    assert!(matches!(removed, Err(BridgeError::Busy)));
    assert!(matches!(preview, Err(BridgeError::Busy)));
    assert!(downloads.join("a.zip").is_file());
    assert!(!Samples::paths(dir.path()).settings().exists());
    drop(ticket);
    session.download_remove("a.zip").unwrap();
    assert!(!downloads.join("a.zip").exists());
}
