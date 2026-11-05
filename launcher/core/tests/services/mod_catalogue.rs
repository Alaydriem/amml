use crate::support::{Catalogues, FakeServer, RegistryTree, Route};
use amml_launcher_core::{AmmlLevel, LauncherError, ModId};
use amml_registry::ModDocument;

fn amml() -> amml_registry::AmmlSection {
    RegistryTree::amml(vec![RegistryTree::amml_release("26.60.1", 2, &["1.26.60.5"])])
}

fn doc(namespace: &str, name: &str, core: bool, default: bool, version: &str) -> ModDocument {
    RegistryTree::document(
        namespace,
        name,
        core,
        default,
        vec![RegistryTree::release(version, 2, "https://example.test/x.zip", "aa", 10)],
    )
}

#[test]
fn defaults_include_core_only_from_the_official_registry() {
    let official = FakeServer::start(vec![]);
    RegistryTree::publish(
        &official,
        "Official",
        &amml(),
        &[
            doc("amml", "modmanager", true, false, "26.60.1"),
            doc("alaydriem", "zoom", false, true, "26.60.1"),
            doc("alaydriem", "peek", false, false, "26.60.1"),
        ],
    );
    let third = FakeServer::start(vec![]);
    RegistryTree::publish(
        &third,
        "Third",
        &amml(),
        &[doc("x", "y", true, true, "1.0.0"), doc("x", "evil", true, false, "1.0.0")],
    );
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogues::catalogue(
        vec![
            Catalogues::client(&official, true, &cache.path().join("1")),
            Catalogues::client(&third, false, &cache.path().join("2")),
        ],
        downloads.path(),
    );
    catalogue.registries_mut().refresh_all();

    let found: Vec<(String, bool)> = catalogue
        .defaults_for(2, &AmmlLevel::Local)
        .unwrap()
        .mods
        .iter()
        .map(|resolved| (resolved.document.id.clone(), resolved.core))
        .collect();
    assert!(found.contains(&("@amml/modmanager".to_string(), true)));
    assert!(found.contains(&("@alaydriem/zoom".to_string(), false)));
    assert!(found.contains(&("@x/y".to_string(), false)));
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(catalogue.defaults_for(9, &AmmlLevel::Local).unwrap().mods.is_empty());
}

#[test]
fn the_first_registry_with_the_mod_wins() {
    let first = FakeServer::start(vec![]);
    RegistryTree::publish(&first, "First", &amml(), &[doc("x", "y", false, false, "1.0.0")]);
    let second = FakeServer::start(vec![]);
    RegistryTree::publish(
        &second,
        "Second",
        &amml(),
        &[doc("x", "y", false, true, "2.0.0"), doc("x", "z", false, true, "2.0.0")],
    );
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogues::catalogue(
        vec![
            Catalogues::client(&first, true, &cache.path().join("1")),
            Catalogues::client(&second, false, &cache.path().join("2")),
        ],
        downloads.path(),
    );
    catalogue.registries_mut().refresh_all();

    let resolved = catalogue.resolve(&ModId::parse("@x/y").unwrap(), 2, &AmmlLevel::Local).unwrap();
    assert_eq!(resolved.release.version, "1.0.0");
    assert_eq!(resolved.registry.base, first.base());
    // a mod only the second registry has is found there
    assert_eq!(
        catalogue.resolve(&ModId::parse("@x/z").unwrap(), 2, &AmmlLevel::Local).unwrap().registry.base,
        second.base()
    );
    assert!(matches!(
        catalogue.resolve(&ModId::parse("@x/none").unwrap(), 2, &AmmlLevel::Local),
        Err(LauncherError::UnknownMod(_))
    ));
    assert!(matches!(
        catalogue.resolve(&ModId::parse("@x/y").unwrap(), 5, &AmmlLevel::Local),
        Err(LauncherError::UnknownMod(_))
    ));

    // the first registry lists @x/y without `default`, so the second registry's default flag does not count
    let defaults: Vec<String> = catalogue
        .defaults_for(2, &AmmlLevel::Local)
        .unwrap()
        .mods
        .iter()
        .map(|resolved| resolved.document.id.clone())
        .collect();
    assert_eq!(defaults, vec!["@x/z".to_string()]);
}

#[test]
fn a_stale_document_still_installs_when_the_zip_hash_matches() {
    let registry = FakeServer::start(vec![]);
    let zips = FakeServer::start(vec![]);
    let work = tempfile::tempdir().unwrap();
    let release = Catalogues::serve_mod(&zips, work.path(), "x", "y", "1.0.0", 2, &[]);
    let document = RegistryTree::document("x", "y", false, true, vec![release]);
    RegistryTree::publish(&registry, "Reg", &amml(), &[document]);
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let id = ModId::parse("@x/y").unwrap();

    let mut online = Catalogues::catalogue(vec![Catalogues::client(&registry, false, cache.path())], downloads.path());
    assert!(!online.resolve(&id, 2, &AmmlLevel::Local).unwrap().stale);
    let base = registry.base();
    drop(registry);

    let mut offline = Catalogues::catalogue(
        vec![Catalogues::client_for(amml_launcher_core::RegistryRef::new(base), cache.path())],
        downloads.path(),
    );
    let resolved = offline.resolve(&id, 2, &AmmlLevel::Local).unwrap();
    assert!(resolved.stale);
    let file = offline.fetch(&resolved, &mut |_| {}).unwrap();
    assert!(file.path.is_file());
    assert_eq!(file.sha256, resolved.release.sha256);
}

#[test]
fn a_stale_document_with_a_changed_zip_is_refused() {
    let registry = FakeServer::start(vec![]);
    let zips = FakeServer::start(vec![]);
    let work = tempfile::tempdir().unwrap();
    let release = Catalogues::serve_mod(&zips, work.path(), "x", "y", "1.0.0", 2, &[]);
    let path = "/zips/x-y-1.0.0.zip";
    let mut changed = std::fs::read(work.path().join("x-y-1.0.0.zip")).unwrap();
    let last = changed.len() - 1;
    changed[last] ^= 0xff;
    zips.replace(path, Route::new(changed));
    let document = RegistryTree::document("x", "y", false, true, vec![release]);
    RegistryTree::publish(&registry, "Reg", &amml(), &[document]);
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let id = ModId::parse("@x/y").unwrap();
    let mut online = Catalogues::catalogue(vec![Catalogues::client(&registry, false, cache.path())], downloads.path());
    online.resolve(&id, 2, &AmmlLevel::Local).unwrap();
    let base = registry.base();
    drop(registry);

    let mut offline = Catalogues::catalogue(
        vec![Catalogues::client_for(amml_launcher_core::RegistryRef::new(base), cache.path())],
        downloads.path(),
    );
    let resolved = offline.resolve(&id, 2, &AmmlLevel::Local).unwrap();
    assert!(resolved.stale);
    assert!(matches!(offline.fetch(&resolved, &mut |_| {}), Err(LauncherError::ChecksumMismatch { .. })));
    assert_eq!(std::fs::read_dir(downloads.path()).unwrap().count(), 0);
}

fn gated(old_min: Option<&str>, new_min: Option<&str>) -> ModDocument {
    RegistryTree::document(
        "x",
        "g",
        false,
        true,
        vec![RegistryTree::release_min("1.0.0", 2, old_min), RegistryTree::release_min("2.0.0", 2, new_min)],
    )
}

fn one_registry(
    document: ModDocument,
    root: &std::path::Path,
    downloads: &std::path::Path,
) -> (FakeServer, amml_launcher_core::ModCatalogue<amml_launcher_core::UreqHttpFetch>) {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Reg", &amml(), &[document]);
    let mut catalogue = Catalogues::catalogue(vec![Catalogues::client(&server, true, root)], downloads);
    catalogue.registries_mut().refresh_all();
    (server, catalogue)
}

#[test]
fn resolve_picks_newest_fitting_release() {
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let (_server, mut catalogue) = one_registry(gated(None, Some("26.62.0")), cache.path(), downloads.path());
    let id = ModId::parse("@x/g").unwrap();

    let old = AmmlLevel::Release(amml_registry::ReleaseVersion::parse("26.60.5").unwrap());
    assert_eq!(catalogue.resolve(&id, 2, &old).unwrap().release.version, "1.0.0");
    let new = AmmlLevel::Release(amml_registry::ReleaseVersion::parse("26.62.0").unwrap());
    assert_eq!(catalogue.resolve(&id, 2, &new).unwrap().release.version, "2.0.0");
    assert_eq!(catalogue.resolve(&id, 2, &AmmlLevel::Local).unwrap().release.version, "2.0.0");
}

#[test]
fn resolve_errors_needs_amml_when_none_fits() {
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let document =
        RegistryTree::document("x", "g", false, true, vec![RegistryTree::release_min("1.0.0", 2, Some("26.60.1"))]);
    let (_server, mut catalogue) = one_registry(document, cache.path(), downloads.path());
    let id = ModId::parse("@x/g").unwrap();
    let old = AmmlLevel::Release(amml_registry::ReleaseVersion::parse("26.60.0").unwrap());

    for result in [catalogue.resolve(&id, 2, &old), catalogue.resolve(&id, 2, &AmmlLevel::Unknown)] {
        match result {
            Err(LauncherError::NeedsAmml { id, needs }) => {
                assert_eq!(id, "@x/g");
                assert_eq!(needs.version, "26.60.1");
                // the official root lists 26.60.1 with this game
                assert_eq!(needs.game.as_deref(), Some("1.26.60.5"));
            }
            other => panic!("{other:?}"),
        }
    }
    let base = catalogue.registries_mut().clients_mut().first().unwrap().registry().base.clone();
    assert!(matches!(catalogue.resolve_in(&base, &id, 2, &old), Err(LauncherError::NeedsAmml { .. })));
    assert_eq!(
        LauncherError::NeedsAmml {
            id: "@x/g".into(),
            needs: amml_launcher_core::NeedsAmml { version: "26.60.1".into(), game: None }
        }
        .to_string(),
        "@x/g needs AMML 26.60.1"
    );
}

#[test]
fn defaults_for_skips_mod_with_no_fitting_release() {
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let document =
        RegistryTree::document("x", "g", false, true, vec![RegistryTree::release_min("1.0.0", 2, Some("26.62.0"))]);
    let (_server, mut catalogue) = one_registry(document, cache.path(), downloads.path());
    let old = AmmlLevel::Release(amml_registry::ReleaseVersion::parse("26.60.5").unwrap());
    let skipped = catalogue.defaults_for(2, &old).unwrap();
    assert!(skipped.mods.is_empty());
    assert_eq!(skipped.skipped, vec![("@x/g".to_string(), "it needs AMML 26.62.0".to_string())]);
    assert_eq!(catalogue.defaults_for(2, &AmmlLevel::Local).unwrap().mods.len(), 1);
}
