use crate::support::{Catalogues, FakeServer, RegistryTree, Route};
use amml_launcher_core::{LauncherError, ModId};
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
        .defaults_for(2)
        .unwrap()
        .mods
        .iter()
        .map(|resolved| (resolved.document.id.clone(), resolved.core))
        .collect();
    assert!(found.contains(&("@amml/modmanager".to_string(), true)));
    assert!(found.contains(&("@alaydriem/zoom".to_string(), false)));
    assert!(found.contains(&("@x/y".to_string(), false)));
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(catalogue.defaults_for(9).unwrap().mods.is_empty());
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

    let resolved = catalogue.resolve(&ModId::parse("@x/y").unwrap(), 2).unwrap();
    assert_eq!(resolved.release.version, "1.0.0");
    assert_eq!(resolved.registry.base, first.base());
    // a mod only the second registry has is found there
    assert_eq!(catalogue.resolve(&ModId::parse("@x/z").unwrap(), 2).unwrap().registry.base, second.base());
    assert!(matches!(catalogue.resolve(&ModId::parse("@x/none").unwrap(), 2), Err(LauncherError::UnknownMod(_))));
    assert!(matches!(catalogue.resolve(&ModId::parse("@x/y").unwrap(), 5), Err(LauncherError::UnknownMod(_))));

    // the first registry lists @x/y without `default`, so the second registry's default flag does not count
    let defaults: Vec<String> =
        catalogue.defaults_for(2).unwrap().mods.iter().map(|resolved| resolved.document.id.clone()).collect();
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
    assert!(!online.resolve(&id, 2).unwrap().stale);
    let base = registry.base();
    drop(registry);

    let mut offline = Catalogues::catalogue(
        vec![Catalogues::client_for(amml_launcher_core::RegistryRef::new(base), cache.path())],
        downloads.path(),
    );
    let resolved = offline.resolve(&id, 2).unwrap();
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
    online.resolve(&id, 2).unwrap();
    let base = registry.base();
    drop(registry);

    let mut offline = Catalogues::catalogue(
        vec![Catalogues::client_for(amml_launcher_core::RegistryRef::new(base), cache.path())],
        downloads.path(),
    );
    let resolved = offline.resolve(&id, 2).unwrap();
    assert!(resolved.stale);
    assert!(matches!(offline.fetch(&resolved, &mut |_| {}), Err(LauncherError::ChecksumMismatch { .. })));
    assert_eq!(std::fs::read_dir(downloads.path()).unwrap().count(), 0);
}
