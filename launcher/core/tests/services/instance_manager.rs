use crate::support::{Catalogues, FakeServer, Harness, InstanceFolder, RegistryTree, Route};
use amml_gdk::GameVersion;
use amml_launcher_core::{
    AmmlLevel, CoreProgress, DataChoice, HclText, InstanceReader, LauncherBlockStore, LauncherError, LauncherModEntry,
    ModChange, ModId, ModSource, NewInstanceRequest, UrlInspector,
};

fn game() -> GameVersion {
    GameVersion::parse("1.26.60.5").unwrap()
}

fn request(harness: &Harness, name: &str) -> NewInstanceRequest {
    NewInstanceRequest {
        game: game(),
        preview: false,
        name: name.to_string(),
        amml: None,
        data: DataChoice::Shared(harness.paths.default_data_dir()),
    }
}

fn standard(harness: &Harness) {
    let amml = harness.amml_release("26.60.1", 2, &["1.26.60.5"]);
    let docs = [
        harness.mod_document("amml", "modmanager", true, false, "26.60.1", 2),
        harness.mod_document("alaydriem", "zoom", false, true, "26.60.1", 2),
        harness.mod_document("alaydriem", "peek", false, false, "26.60.1", 2),
    ];
    harness.publish(vec![amml], &docs);
}

fn existing_instance(harness: &Harness, name: &str, amml_version: &str) -> std::path::PathBuf {
    let dir = harness.paths.instance(name);
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05").with_amml(Some(amml_version));
    dir
}

#[test]
fn a_new_instance_gets_amml_core_and_default_mods() {
    let harness = Harness::new();
    standard(&harness);
    let mut steps = Vec::new();
    let outcome = harness.manager().create(&request(&harness, "test-1"), &mut |step| steps.push(step)).unwrap();

    assert!(outcome.version_installed);
    assert!(outcome.skipped_mods.is_empty());
    assert_eq!(harness.installer.calls(), 1);
    let state = &outcome.state;
    assert_eq!(state.amml.version.as_deref(), Some("26.60.1"));
    assert!(state.managed);
    let ids: Vec<String> = state.mods.iter().map(|installed| installed.id.as_str()).collect();
    assert_eq!(ids, vec!["@alaydriem/zoom", "@amml/modmanager"]);
    let official = ModSource::Registry(harness.server.base());
    assert!(state.mods.iter().all(|installed| installed.enabled && installed.source == official));

    let block = LauncherBlockStore::read(&state.folder).unwrap();
    assert_eq!(block.amml.as_deref(), Some("26.60.1"));
    assert_eq!(block.mods["@amml/modmanager"].source, official);
    assert_eq!(block.mods["@alaydriem/zoom"].version.as_deref(), Some("26.60.1"));
    assert_eq!(state.data_dir, Some(harness.paths.default_data_dir()));
    let version_json = std::fs::read_to_string(state.folder.join("version.json")).unwrap();
    assert!(version_json.contains("\"enableIsolation\": false"), "{version_json}");
    assert!(state.folder.join("Minecraft.Windows.exe.bak").is_file());
    assert!(harness.downloads_empty());
    assert!(steps.contains(&CoreProgress::CreatingInstance) && steps.contains(&CoreProgress::Done));
    assert!(steps.iter().any(|step| matches!(step, CoreProgress::InstallingMod { id } if id == "@amml/modmanager")));
}

#[test]
fn an_unsupported_game_is_refused_before_any_change() {
    let harness = Harness::new();
    standard(&harness);
    let mut asked = request(&harness, "test-1");
    asked.game = GameVersion::parse("1.26.70.1").unwrap();
    let result = harness.manager().create(&asked, &mut |_| {});
    assert!(matches!(result, Err(LauncherError::NotSupported { game }) if game == "1.26.70.1"));
    assert_eq!(harness.installer.calls(), 0);
    assert!(!harness.paths.gdk().instances().exists());
    assert!(!harness.paths.gdk().versions().exists());

    let mut pinned = request(&harness, "test-1");
    pinned.amml = Some("26.59.0".to_string());
    assert!(matches!(harness.manager().create(&pinned, &mut |_| {}), Err(LauncherError::NotSupported { .. })));
    assert_eq!(harness.installer.calls(), 0);
}

#[test]
fn an_installed_version_is_not_installed_again() {
    let harness = Harness::new();
    standard(&harness);
    InstanceFolder::game_files(&harness.paths.gdk().versions().join("1.26.60.05"));
    let outcome = harness.manager().create(&request(&harness, "test-1"), &mut |_| {}).unwrap();
    assert!(!outcome.version_installed);
    assert_eq!(harness.installer.calls(), 0);
}

#[test]
fn a_taken_or_bad_name_is_refused_before_any_download() {
    let harness = Harness::new();
    standard(&harness);
    existing_instance(&harness, "taken", "26.60.0");
    assert!(matches!(harness.manager().create(&request(&harness, "taken"), &mut |_| {}), Err(LauncherError::Gdk(_))));
    assert!(matches!(
        harness.manager().create(&request(&harness, "bad name"), &mut |_| {}),
        Err(LauncherError::Gdk(_))
    ));
    assert_eq!(harness.installer.calls(), 0);
}

#[test]
fn a_core_mod_failure_removes_the_instance_but_keeps_the_version() {
    let harness = Harness::new();
    standard(&harness);
    harness.corrupt_zip("amml-modmanager-26.60.1.zip");
    let result = harness.manager().create(&request(&harness, "test-1"), &mut |_| {});
    assert!(matches!(result, Err(LauncherError::ChecksumMismatch { .. })), "{result:?}");
    assert!(!harness.paths.instance("test-1").exists());
    assert!(harness.paths.gdk().versions().join("1.26.60.05").is_dir());
    assert!(harness.downloads_empty());
}

#[test]
fn an_amml_failure_removes_the_instance() {
    let harness = Harness::new();
    standard(&harness);
    harness.corrupt_zip("amml-26.60.1.zip");
    let result = harness.manager().create(&request(&harness, "test-1"), &mut |_| {});
    assert!(matches!(result, Err(LauncherError::ChecksumMismatch { .. })), "{result:?}");
    assert!(!harness.paths.instance("test-1").exists());
}

#[test]
fn a_default_mod_failure_is_reported_and_the_instance_kept() {
    let harness = Harness::new();
    standard(&harness);
    harness.corrupt_zip("alaydriem-zoom-26.60.1.zip");
    let outcome = harness.manager().create(&request(&harness, "test-1"), &mut |_| {}).unwrap();
    assert_eq!(outcome.skipped_mods.len(), 1);
    assert_eq!(outcome.skipped_mods[0].0, "@alaydriem/zoom");
    assert!(outcome.skipped_mods[0].1.contains("checksum"), "{:?}", outcome.skipped_mods);
    let ids: Vec<String> = outcome.state.mods.iter().map(|installed| installed.id.as_str()).collect();
    assert_eq!(ids, vec!["@amml/modmanager"]);
    assert!(harness.paths.instance("test-1").is_dir());
    assert!(harness.downloads_empty());
}

#[test]
fn delete_needs_the_exact_name_and_never_touches_shared_data() {
    let harness = Harness::new();
    let dir = existing_instance(&harness, "one", "26.60.1");
    let shared = harness.paths.default_data_dir().join("Minecraft Bedrock");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::write(shared.join("x"), b"world").unwrap();
    // the instance points at the shared folder
    std::fs::write(
        dir.join("amml").join("instance.hcl"),
        format!("data_dir = {}\n", HclText::quote(&harness.paths.default_data_dir().to_string_lossy())),
    )
    .unwrap();
    let manager = harness.manager();

    assert!(matches!(manager.delete("one", "two"), Err(LauncherError::ConfirmMismatch { .. })));
    assert!(matches!(manager.delete("one", ""), Err(LauncherError::ConfirmMismatch { .. })));
    assert!(dir.is_dir());

    manager.delete("one", "one").unwrap();
    assert!(!dir.exists());
    assert_eq!(std::fs::read(shared.join("x")).unwrap(), b"world");

    for bad in ["..\\x", "../x", ".staging-1", ".hidden", "", "a\\b"] {
        assert!(matches!(manager.delete(bad, bad), Err(LauncherError::Gdk(_))), "{bad}");
    }
    assert!(harness.paths.default_data_dir().is_dir());
}

#[test]
fn delete_of_a_running_instance_is_refused() {
    let harness = Harness::new();
    let dir = existing_instance(&harness, "one", "26.60.1");
    harness.processes.start(5, dir.join("Minecraft.Windows.exe"));
    let before = InstanceFolder::hashes(&dir);
    assert!(matches!(harness.manager().delete("one", "one"), Err(LauncherError::InstanceRunning(_))));
    assert_eq!(InstanceFolder::hashes(&dir), before);
    assert!(matches!(harness.manager().delete("missing", "missing"), Err(LauncherError::NotAnInstance(_))));
}

#[test]
fn update_amml_replaces_the_loader_and_records_the_version() {
    let harness = Harness::new();
    let amml_old = harness.amml_release("26.60.0", 2, &["1.26.60.5"]);
    let amml_new = harness.amml_release("26.60.1", 2, &["1.26.60.5"]);
    harness.publish(vec![amml_new, amml_old], &[]);
    let dir = existing_instance(&harness, "one", "26.60.0");
    LauncherBlockStore::set_amml(&dir, "26.60.0").unwrap();

    let version = harness.manager().update_amml("one", None, &mut |_| {}).unwrap();
    assert_eq!(version, "26.60.1");
    assert_eq!(std::fs::read(dir.join("amml.dll")).unwrap(), b"loader 26.60.1");
    assert_eq!(LauncherBlockStore::read(&dir).unwrap().amml.as_deref(), Some("26.60.1"));
    assert_eq!(InstanceReader::read(&dir).unwrap().amml.version.as_deref(), Some("26.60.1"));
    assert!(harness.downloads_empty());

    let back = harness.manager().update_amml("one", Some("26.60.0".to_string()), &mut |_| {}).unwrap();
    assert_eq!(back, "26.60.0");
    assert_eq!(std::fs::read(dir.join("amml.dll")).unwrap(), b"loader 26.60.0");
}

#[test]
fn update_amml_of_a_running_instance_changes_nothing() {
    let harness = Harness::new();
    harness.publish(vec![harness.amml_release("26.60.1", 2, &["1.26.60.5"])], &[]);
    let dir = existing_instance(&harness, "one", "26.60.0");
    harness.processes.start(5, dir.join("Minecraft.Windows.exe"));
    let before = InstanceFolder::hashes(&dir);
    assert!(matches!(harness.manager().update_amml("one", None, &mut |_| {}), Err(LauncherError::InstanceRunning(_))));
    assert_eq!(InstanceFolder::hashes(&dir), before);
}

#[test]
fn a_registry_mod_installs_and_an_inspected_zip_needs_consent() {
    let harness = Harness::new();
    standard(&harness);
    let dir = existing_instance(&harness, "one", "26.60.1");
    let mut manager = harness.manager();

    let id = ModId::parse("@alaydriem/peek").unwrap();
    let resolved = manager.catalogue_mut().resolve(&id, 2, &AmmlLevel::Local).unwrap();
    assert_eq!(manager.install_mod("one", &resolved, &mut |_| {}).unwrap(), ModChange::Installed);
    assert_eq!(
        manager.install_mod("one", &resolved, &mut |_| {}).unwrap(),
        ModChange::Updated { from: Some("26.60.1".to_string()) }
    );
    assert!(dir.join("mods").join("amml").join("@alaydriem").join("peek").join("amml.hcl").is_file());

    let url = harness.server.url("/zips/alaydriem-zoom-26.60.1.zip");
    let inspection = UrlInspector::inspect(manager.catalogue_mut(), &url, &mut |_| {}).unwrap();
    assert!(inspection.matched.is_some());
    assert_eq!(manager.install_inspected("one", inspection, false).unwrap(), ModChange::Installed);

    let mut third = harness.manager();
    let unlisted = third_zip(&harness);
    let inspection = UrlInspector::inspect(third.catalogue_mut(), &unlisted, &mut |_| {}).unwrap();
    assert!(inspection.matched.is_none());
    assert!(matches!(
        third.install_inspected("one", inspection.clone(), false),
        Err(LauncherError::UnofficialNeedsConsent { .. })
    ));
    assert!(harness.downloads_empty());
}

fn third_zip(harness: &Harness) -> String {
    let work = tempfile::tempdir().unwrap();
    let zip = crate::support::Zips::mod_zip(&work.path().join("o.zip"), "own", "thing", "1.0.0", 2, &[]);
    harness.server.replace("/zips/own-thing.zip", crate::support::Route::new(std::fs::read(zip).unwrap()));
    harness.server.url("/zips/own-thing.zip")
}

fn failing(status: u16) -> Route {
    let mut route = Route::new(Vec::new());
    route.status = Some(status);
    route
}

#[test]
fn a_mod_release_for_another_api_is_refused_before_any_download() {
    let harness = Harness::new();
    let docs = [harness.mod_document("alaydriem", "fly", false, true, "26.61.0", 3)];
    harness.publish(vec![harness.amml_release("26.60.1", 2, &["1.26.60.5"])], &docs);
    let dir = existing_instance(&harness, "one", "26.60.1");
    let before = InstanceFolder::hashes(&dir);
    let mut manager = harness.manager();
    let resolved =
        manager.catalogue_mut().resolve(&ModId::parse("@alaydriem/fly").unwrap(), 3, &AmmlLevel::Local).unwrap();
    match manager.install_mod("one", &resolved, &mut |_| {}) {
        Err(LauncherError::ApiMismatch { mod_api, amml_api, .. }) => assert_eq!((mod_api, amml_api), (3, 2)),
        other => panic!("{other:?}"),
    }
    assert_eq!(InstanceFolder::hashes(&dir), before);
    assert_eq!(harness.server.count("/zips/alaydriem-fly-26.61.0.zip"), 0);
}

#[test]
fn a_zip_that_holds_another_mod_than_the_one_asked_for_is_refused() {
    let harness = Harness::new();
    let real = harness.mod_document("amml", "modmanager", true, false, "26.60.1", 2);
    let stolen = RegistryTree::document("x", "y", false, true, real.versions.clone());
    harness.publish(vec![harness.amml_release("26.60.1", 2, &["1.26.60.5"])], &[real, stolen]);
    let dir = existing_instance(&harness, "one", "26.60.1");
    InstanceFolder { dir: dir.clone() }.with_mod(true, "amml", "modmanager", Some("26.60.0"), 2);
    let before = InstanceFolder::hashes(&dir);
    let mut manager = harness.manager();
    let resolved = manager.catalogue_mut().resolve(&ModId::parse("@x/y").unwrap(), 2, &AmmlLevel::Local).unwrap();
    match manager.install_mod("one", &resolved, &mut |_| {}) {
        Err(LauncherError::BadArchive { reason, .. }) => {
            assert!(reason.contains("@amml/modmanager") && reason.contains("@x/y"), "{reason}")
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(InstanceFolder::hashes(&dir), before);
    assert!(harness.downloads_empty());
}

#[test]
fn a_failed_official_index_page_fails_create_and_removes_the_instance() {
    let harness = Harness::new();
    standard(&harness);
    harness.server.replace("/index/api-2/0001.json", failing(500));
    let result = harness.manager().create(&request(&harness, "test-1"), &mut |_| {});
    assert!(matches!(result, Err(LauncherError::Http { .. })), "{result:?}");
    assert!(!harness.paths.instance("test-1").exists());
    assert!(harness.paths.gdk().versions().join("1.26.60.05").is_dir());
}

#[test]
fn a_failed_core_document_fails_create_and_removes_the_instance() {
    let harness = Harness::new();
    standard(&harness);
    harness.server.replace("/mods/@amml/modmanager.json", failing(500));
    let result = harness.manager().create(&request(&harness, "test-1"), &mut |_| {});
    assert!(matches!(result, Err(LauncherError::Http { .. })), "{result:?}");
    assert!(!harness.paths.instance("test-1").exists());
}

#[test]
fn a_failed_default_document_is_a_skipped_mod() {
    let harness = Harness::new();
    standard(&harness);
    harness.server.replace("/mods/@alaydriem/zoom.json", failing(500));
    let outcome = harness.manager().create(&request(&harness, "test-1"), &mut |_| {}).unwrap();
    assert_eq!(outcome.skipped_mods.len(), 1);
    assert_eq!(outcome.skipped_mods[0].0, "@alaydriem/zoom");
    assert_eq!(outcome.state.mods.len(), 1);
}

#[test]
fn a_failed_index_page_stops_adding_core_mods_to_an_instance() {
    let harness = Harness::new();
    standard(&harness);
    harness.server.replace("/index/api-2/0001.json", failing(500));
    let dir = existing_instance(&harness, "one", "26.60.1");
    let before = InstanceFolder::hashes(&dir);
    assert!(harness.manager().ensure_core_mods("one", &mut |_| {}).is_err());
    assert_eq!(InstanceFolder::hashes(&dir), before);
}

#[test]
fn a_core_mod_stays_core_when_the_registry_is_down_because_install_recorded_it() {
    let harness = Harness::new();
    standard(&harness);
    let dir = existing_instance(&harness, "one", "26.60.1");
    let mut manager = harness.manager();
    let resolved =
        manager.catalogue_mut().resolve(&ModId::parse("@amml/modmanager").unwrap(), 2, &AmmlLevel::Local).unwrap();
    manager.install_mod("one", &resolved, &mut |_| {}).unwrap();
    assert_eq!(LauncherBlockStore::read(&dir).unwrap().mods["@amml/modmanager"].core, Some(true));

    let mut dead = Route::new(Vec::new());
    dead.fail = true;
    harness.server.replace("/mods/@amml/modmanager.json", dead);
    std::fs::remove_dir_all(harness.paths.registry_cache()).unwrap();
    let before = InstanceFolder::hashes(&dir);
    let id = ModId::parse("@amml/modmanager").unwrap();
    assert!(matches!(manager.set_mod_enabled("one", &id, false), Err(LauncherError::CoreMod(_))));
    assert!(matches!(manager.remove_mod("one", &id), Err(LauncherError::CoreMod(_))));
    assert_eq!(InstanceFolder::hashes(&dir), before);
}

#[test]
fn an_official_mod_with_no_record_cannot_be_judged_while_the_registry_is_down() {
    let harness = Harness::new();
    standard(&harness);
    let dir = existing_instance(&harness, "one", "26.60.1");
    InstanceFolder { dir: dir.clone() }.with_mod(true, "amml", "modmanager", Some("26.60.1"), 2).with_mod(
        true,
        "me",
        "mine",
        Some("1.0.0"),
        2,
    );
    let entry = |source: ModSource| LauncherModEntry { version: Some("1.0.0".to_string()), source, core: None };
    LauncherBlockStore::set_mod(&dir, "@amml/modmanager", entry(ModSource::Registry(harness.server.base()))).unwrap();
    LauncherBlockStore::set_mod(&dir, "@me/mine", entry(ModSource::Local)).unwrap();
    let mut dead = Route::new(Vec::new());
    dead.fail = true;
    harness.server.replace("/mods/@amml/modmanager.json", dead.clone());
    harness.server.replace("/mods/@me/mine.json", dead);
    let _ = std::fs::remove_dir_all(harness.paths.registry_cache());
    let mut manager = harness.manager();

    let before = InstanceFolder::hashes(&dir);
    let official = ModId::parse("@amml/modmanager").unwrap();
    assert!(matches!(manager.set_mod_enabled("one", &official, false), Err(LauncherError::Offline { .. })));
    assert!(matches!(manager.remove_mod("one", &official), Err(LauncherError::Offline { .. })));
    assert_eq!(InstanceFolder::hashes(&dir), before);

    // a mod that never came from the official registry is not blocked by its being down
    manager.remove_mod("one", &ModId::parse("@me/mine").unwrap()).unwrap();
}

#[test]
fn an_update_resolves_from_the_registry_the_mod_came_from() {
    let harness = Harness::new();
    let first = harness.mod_document("x", "y", false, true, "26.60.9", 2);
    harness.publish(vec![harness.amml_release("26.60.1", 2, &["1.26.60.5"])], &[first]);
    let second = FakeServer::start(vec![]);
    let other_release = RegistryTree::release("26.60.5", 2, "https://example.test/y.zip", "aa", 1);
    RegistryTree::publish(
        &second,
        "Second",
        &RegistryTree::amml(vec![]),
        &[RegistryTree::document("x", "y", false, true, vec![other_release])],
    );
    existing_instance(&harness, "one", "26.60.1");
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let catalogue = Catalogues::catalogue(
        vec![
            Catalogues::client_for(harness.official(), &cache.path().join("1")),
            Catalogues::client(&second, false, &cache.path().join("2")),
        ],
        downloads.path(),
    );
    let mut manager = harness.manager_with(catalogue);
    let id = ModId::parse("@x/y").unwrap();

    let from_second = manager.resolve_mod("one", &id, Some(&ModSource::Registry(second.base()))).unwrap();
    assert_eq!(
        (from_second.registry.base.as_str(), from_second.release.version.as_str()),
        (second.base().as_str(), "26.60.5")
    );
    let anywhere = manager.resolve_mod("one", &id, None).unwrap();
    assert_eq!(anywhere.release.version, "26.60.9");
    let local = manager.resolve_mod("one", &id, Some(&ModSource::Local)).unwrap();
    assert_eq!(local.release.version, "26.60.9");
    let gone = ModSource::Registry("https://gone.example/".to_string());
    assert!(matches!(manager.resolve_mod("one", &id, Some(&gone)), Err(LauncherError::UnknownMod(_))));
}

#[test]
fn a_name_with_a_leading_dot_is_refused_by_create() {
    let harness = Harness::new();
    standard(&harness);
    let result = harness.manager().create(&request(&harness, ".foo"), &mut |_| {});
    assert!(matches!(result, Err(LauncherError::Gdk(_))), "{result:?}");
    assert_eq!(harness.installer.calls(), 0);
    assert!(!harness.paths.gdk().instances().exists());
}

#[test]
fn install_mod_works_on_local_build_without_mods() {
    let harness = Harness::new();
    standard(&harness);
    let dir = harness.paths.instance("one");
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05").with_amml(Some("26.60.5+local")).with_amml_api("2");
    let mut manager = harness.manager();

    let resolved =
        manager.catalogue_mut().resolve(&ModId::parse("@alaydriem/peek").unwrap(), 2, &AmmlLevel::Local).unwrap();

    assert_eq!(manager.install_mod("one", &resolved, &mut |_| {}).unwrap(), ModChange::Installed);
}

#[test]
fn install_into_instance_uses_instance_amml_level() {
    let harness = Harness::new();
    let older = harness.mod_document("alaydriem", "gate", false, false, "1.0.0", 2).versions.remove(0);
    let mut newer = harness.mod_document("alaydriem", "gate", false, false, "2.0.0", 2).versions.remove(0);
    newer.min_amml = Some("26.62.0".to_string());
    let document = RegistryTree::document("alaydriem", "gate", false, false, vec![older, newer]);
    harness.publish(
        vec![harness.amml_release("26.60.5", 2, &["1.26.60.5"]), harness.amml_release("26.62.0", 2, &["1.26.62.1"])],
        &[document],
    );
    let dir = existing_instance(&harness, "one", "26.60.5");
    let mut manager = harness.manager();
    let id = ModId::parse("@alaydriem/gate").unwrap();

    let resolved = manager.resolve_mod("one", &id, None).unwrap();
    assert_eq!(resolved.release.version, "1.0.0");
    assert_eq!(manager.install_mod("one", &resolved, &mut |_| {}).unwrap(), ModChange::Installed);
    assert!(dir.join("mods").join("amml").join("@alaydriem").join("gate").join("amml.hcl").is_file());
    assert_eq!(harness.server.count("/zips/alaydriem-gate-2.0.0.zip"), 0);

    let newer_instance = existing_instance(&harness, "two", "26.62.0");
    assert!(newer_instance.is_dir());
    assert_eq!(manager.resolve_mod("two", &id, None).unwrap().release.version, "2.0.0");
}
