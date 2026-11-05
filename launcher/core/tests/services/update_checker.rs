use crate::support::{Catalogues, FakeServer, InstanceFolder, RegistryTree};
use amml_launcher_core::{InstanceReader, UpdateChecker, UpdateRow, UpdateStatus};
use amml_registry::{AmmlSection, ModRelease};
use std::path::Path;

fn amml(releases: &[(&str, u32)]) -> AmmlSection {
    RegistryTree::amml(
        releases.iter().map(|(version, api)| RegistryTree::amml_release(version, *api, &["1.26.60.5"])).collect(),
    )
}

fn doc(name: &str, releases: &[(&str, u32)]) -> amml_registry::ModDocument {
    let releases: Vec<ModRelease> = releases
        .iter()
        .map(|(version, api)| RegistryTree::release(version, *api, "https://example.test/x.zip", "aa", 1))
        .collect();
    RegistryTree::document("a", name, false, true, releases)
}

fn instance(
    dir: &Path,
    base: &str,
    amml_version: &str,
    mods: &[(&str, Option<&str>, &str)],
) -> amml_launcher_core::InstanceState {
    let mut folder = InstanceFolder::create(dir, "1.26.60.05").with_amml(Some(amml_version));
    let mut entries = Vec::new();
    for (name, version, source) in mods {
        folder = folder.with_mod(true, "a", name, *version, 2);
        let source = if *source == "registry" { base.to_string() } else { source.to_string() };
        let version_text = version.map(|v| format!("\"version\":\"{v}\",")).unwrap_or_default();
        entries.push(format!("\"@a/{name}\":{{{version_text}\"source\":\"{source}\"}}"));
    }
    folder.with_version_json(&format!(
        "{{\"gameVersion\":\"1.26.60.05\",\"launcher\":{{\"amml\":\"{amml_version}\",\"mods\":{{{}}}}}}}",
        entries.join(",")
    ));
    InstanceReader::read(dir).unwrap()
}

fn row<'a>(rows: &'a [UpdateRow], id: &str) -> &'a UpdateRow {
    rows.iter().find(|row| row.id == id).unwrap()
}

#[test]
fn a_newer_release_for_the_same_api_is_available() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Reg", &amml(&[("26.60.1", 2)]), &[doc("m", &[("26.60.10", 2), ("26.60.9", 2)])]);
    let root = tempfile::tempdir().unwrap();
    let state = instance(&root.path().join("one"), &server.base(), "26.60.1", &[("m", Some("26.60.9"), "registry")]);
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogues::catalogue(vec![Catalogues::client(&server, true, cache.path())], downloads.path());

    let rows = UpdateChecker::check(&mut catalogue, &state);
    let found = row(&rows, "@a/m");
    assert_eq!(found.status, UpdateStatus::Available);
    assert_eq!((found.installed.as_deref(), found.available.as_deref()), (Some("26.60.9"), Some("26.60.10")));
    assert_eq!(row(&rows, UpdateChecker::AMML_ID).status, UpdateStatus::UpToDate);
}

#[test]
fn a_newer_release_only_for_another_api_is_up_to_date() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Reg", &amml(&[("26.60.1", 2)]), &[doc("m", &[("26.60.0", 2), ("26.60.5", 3)])]);
    let root = tempfile::tempdir().unwrap();
    let state = instance(&root.path().join("one"), &server.base(), "26.60.1", &[("m", Some("26.60.0"), "registry")]);
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogues::catalogue(vec![Catalogues::client(&server, true, cache.path())], downloads.path());
    let rows = UpdateChecker::check(&mut catalogue, &state);
    assert_eq!(row(&rows, "@a/m").status, UpdateStatus::UpToDate);
    assert_eq!(row(&rows, "@a/m").available.as_deref(), Some("26.60.0"));
}

#[test]
fn a_source_built_mod_is_unknown() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Reg", &amml(&[("26.60.1", 2)]), &[doc("m", &[("26.60.5", 2)])]);
    let root = tempfile::tempdir().unwrap();
    let state = instance(&root.path().join("one"), &server.base(), "26.60.1", &[("m", None, "local")]);
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogues::catalogue(vec![Catalogues::client(&server, true, cache.path())], downloads.path());
    let rows = UpdateChecker::check(&mut catalogue, &state);
    assert_eq!(row(&rows, "@a/m").status, UpdateStatus::Unknown);
}

#[test]
fn a_local_mod_with_a_version_is_compared_with_every_registry_or_not_listed() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Reg", &amml(&[("26.60.1", 2)]), &[doc("listed", &[("26.60.5", 2)])]);
    let root = tempfile::tempdir().unwrap();
    let state = instance(
        &root.path().join("one"),
        &server.base(),
        "26.60.1",
        &[("listed", Some("26.60.0"), "local"), ("lonely", Some("1.0.0"), "local")],
    );
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogues::catalogue(vec![Catalogues::client(&server, true, cache.path())], downloads.path());
    let rows = UpdateChecker::check(&mut catalogue, &state);
    assert_eq!(row(&rows, "@a/listed").status, UpdateStatus::Available);
    assert_eq!(row(&rows, "@a/lonely").status, UpdateStatus::NotListed);
}

#[test]
fn a_url_mod_is_unofficial_and_never_fetched() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Reg", &amml(&[("26.60.1", 2)]), &[doc("m", &[("26.60.5", 2)])]);
    let root = tempfile::tempdir().unwrap();
    let state = instance(
        &root.path().join("one"),
        &server.base(),
        "26.60.1",
        &[("m", Some("26.60.0"), "url:https://x.example/m.zip")],
    );
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogues::catalogue(vec![Catalogues::client(&server, true, cache.path())], downloads.path());
    server.clear_requests();
    let rows = UpdateChecker::check(&mut catalogue, &state);
    assert_eq!(row(&rows, "@a/m").status, UpdateStatus::Unofficial);
    assert!(server.requests().iter().all(|request| !request.path.contains("/mods/")), "{:?}", server.requests());
}

#[test]
fn amml_on_a_newer_api_is_flagged_as_an_api_change() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Reg", &amml(&[("26.61.0", 3), ("26.60.1", 2)]), &[]);
    let root = tempfile::tempdir().unwrap();
    let state = instance(&root.path().join("one"), &server.base(), "26.60.1", &[]);
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogues::catalogue(vec![Catalogues::client(&server, true, cache.path())], downloads.path());
    let rows = UpdateChecker::check(&mut catalogue, &state);
    let found = row(&rows, UpdateChecker::AMML_ID);
    assert_eq!(found.status, UpdateStatus::Available);
    assert_eq!(found.available.as_deref(), Some("26.61.0"));
    assert!(found.api_change);

    let same_api = instance(&root.path().join("two"), &server.base(), "26.61.0", &[]);
    let rows = UpdateChecker::check(&mut catalogue, &same_api);
    assert!(!row(&rows, UpdateChecker::AMML_ID).api_change);
    assert_eq!(row(&rows, UpdateChecker::AMML_ID).status, UpdateStatus::UpToDate);
}

#[test]
fn a_registry_that_is_down_with_no_cache_is_offline() {
    let server = FakeServer::start(vec![]);
    let base = server.base();
    drop(server);
    let root = tempfile::tempdir().unwrap();
    let state = instance(&root.path().join("one"), &base, "26.60.1", &[("m", Some("26.60.0"), "registry")]);
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let client = Catalogues::client_for(amml_launcher_core::RegistryRef::official_at(base), cache.path());
    let mut catalogue = Catalogues::catalogue(vec![client], downloads.path());
    let rows = UpdateChecker::check(&mut catalogue, &state);
    assert_eq!(row(&rows, "@a/m").status, UpdateStatus::Offline);
    assert_eq!(row(&rows, UpdateChecker::AMML_ID).status, UpdateStatus::Offline);
}

#[test]
fn a_mod_for_the_old_api_is_offered_its_release_for_the_new_api_after_an_amml_update() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(
        &server,
        "Reg",
        &amml(&[("26.61.0", 3), ("26.60.1", 2)]),
        &[doc("m", &[("26.61.0", 3), ("26.60.0", 2)]), doc("old", &[("26.60.0", 2)])],
    );
    let root = tempfile::tempdir().unwrap();
    // AMML was moved to API 3; both mods still hold API 2 builds
    let state = instance(
        &root.path().join("one"),
        &server.base(),
        "26.61.0",
        &[("m", Some("26.60.0"), "registry"), ("old", Some("26.60.0"), "registry")],
    );
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogues::catalogue(vec![Catalogues::client(&server, true, cache.path())], downloads.path());
    let rows = UpdateChecker::check(&mut catalogue, &state);
    let moved = row(&rows, "@a/m");
    assert_eq!((moved.status, moved.available.as_deref()), (UpdateStatus::Available, Some("26.61.0")));
    // no release fits the instance's API, so the mod is not reported as up to date
    let stuck = row(&rows, "@a/old");
    assert_eq!((stuck.status, stuck.available.as_deref()), (UpdateStatus::Unknown, None));
}

fn local_build(dir: &Path) -> amml_launcher_core::InstanceState {
    InstanceFolder::create(dir, "1.26.60.05").with_amml(Some("26.60.5+local")).with_amml_api("2");
    InstanceReader::read(dir).unwrap()
}

#[test]
fn local_build_gets_local_build_status() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Reg", &amml(&[("26.60.5", 2)]), &[]);
    let root = tempfile::tempdir().unwrap();
    let state = local_build(&root.path().join("one"));
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogues::catalogue(vec![Catalogues::client(&server, true, cache.path())], downloads.path());

    let rows = UpdateChecker::check(&mut catalogue, &state);

    let found = row(&rows, UpdateChecker::AMML_ID);
    assert_eq!(found.status, UpdateStatus::LocalBuild);
    assert_eq!(found.available.as_deref(), Some("26.60.5"));
}

#[test]
fn local_build_wins_over_offline() {
    let server = FakeServer::start(vec![]);
    let base = server.base();
    drop(server);
    let root = tempfile::tempdir().unwrap();
    let state = local_build(&root.path().join("one"));
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let client = Catalogues::client_for(amml_launcher_core::RegistryRef::official_at(base), cache.path());
    let mut catalogue = Catalogues::catalogue(vec![client], downloads.path());

    let rows = UpdateChecker::check(&mut catalogue, &state);

    let found = row(&rows, UpdateChecker::AMML_ID);
    assert_eq!(found.status, UpdateStatus::LocalBuild);
    assert_eq!(found.available, None);
}

#[test]
fn local_build_wins_over_not_listed() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(
        &server,
        "Reg",
        &RegistryTree::amml(vec![RegistryTree::amml_release("26.60.5", 2, &["1.26.52.3"])]),
        &[],
    );
    let root = tempfile::tempdir().unwrap();
    let state = local_build(&root.path().join("one"));
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogues::catalogue(vec![Catalogues::client(&server, true, cache.path())], downloads.path());

    let rows = UpdateChecker::check(&mut catalogue, &state);

    let found = row(&rows, UpdateChecker::AMML_ID);
    assert_eq!(found.status, UpdateStatus::LocalBuild);
    assert_eq!(found.available, None);
}
