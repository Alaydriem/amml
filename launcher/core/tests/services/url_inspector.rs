use crate::support::{Catalogues, FakeServer, RegistryTree, Route, Zips};
use amml_launcher_core::{LauncherError, ModSource, UrlInspector};
use amml_registry::HashText;

fn amml() -> amml_registry::AmmlSection {
    RegistryTree::amml(vec![RegistryTree::amml_release("26.60.1", 2, &["1.26.60.5"])])
}

#[test]
fn a_url_matching_a_registry_asset_installs_as_that_registry() {
    let registry = FakeServer::start(vec![]);
    let zips = FakeServer::start(vec![]);
    let work = tempfile::tempdir().unwrap();
    let release = Catalogues::serve_mod(&zips, work.path(), "x", "y", "1.0.0", 2, &[]);
    let url = release.asset.clone();
    RegistryTree::publish(&registry, "Reg", &amml(), &[RegistryTree::document("x", "y", false, true, vec![release])]);
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue =
        Catalogues::catalogue(vec![Catalogues::client(&registry, false, cache.path())], downloads.path());

    let inspection = UrlInspector::inspect(&mut catalogue, &url, &mut |_| {}).unwrap();
    assert_eq!(inspection.id.as_str(), "@x/y");
    assert_eq!(inspection.api_version, 2);
    assert_eq!(inspection.listing.as_ref().unwrap().title, "Y");
    assert_eq!(inspection.matched.as_ref().unwrap().release.version, "1.0.0");
    assert_eq!(inspection.source(false).unwrap(), ModSource::Registry(registry.base()));
}

#[test]
fn a_mirror_with_the_same_bytes_matches_by_hash() {
    let registry = FakeServer::start(vec![]);
    let zips = FakeServer::start(vec![]);
    let work = tempfile::tempdir().unwrap();
    let release = Catalogues::serve_mod(&zips, work.path(), "x", "y", "1.0.0", 2, &[]);
    let bytes = std::fs::read(work.path().join("x-y-1.0.0.zip")).unwrap();
    assert_eq!(HashText::sha256(&bytes), release.sha256);
    zips.replace("/mirror/other.zip", Route::new(bytes));
    RegistryTree::publish(&registry, "Reg", &amml(), &[RegistryTree::document("x", "y", false, true, vec![release])]);
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue =
        Catalogues::catalogue(vec![Catalogues::client(&registry, false, cache.path())], downloads.path());
    let inspection = UrlInspector::inspect(&mut catalogue, &zips.url("/mirror/other.zip"), &mut |_| {}).unwrap();
    assert!(inspection.matched.is_some());
}

#[test]
fn an_unknown_zip_needs_consent_then_is_unofficial() {
    let registry = FakeServer::start(vec![]);
    RegistryTree::publish(&registry, "Reg", &amml(), &[]);
    let zips = FakeServer::start(vec![]);
    let work = tempfile::tempdir().unwrap();
    Catalogues::serve_mod(&zips, work.path(), "x", "y", "1.0.0", 2, &[]);
    let url = zips.url("/zips/x-y-1.0.0.zip");
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue =
        Catalogues::catalogue(vec![Catalogues::client(&registry, false, cache.path())], downloads.path());

    let inspection = UrlInspector::inspect(&mut catalogue, &url, &mut |_| {}).unwrap();
    assert!(inspection.matched.is_none());
    assert!(
        matches!(inspection.source(false), Err(LauncherError::UnofficialNeedsConsent { url: named }) if named == url)
    );
    assert_eq!(inspection.source(true).unwrap(), ModSource::Url(url));
}

#[test]
fn a_zip_with_two_mods_is_refused_and_not_kept() {
    let registry = FakeServer::start(vec![]);
    RegistryTree::publish(&registry, "Reg", &amml(), &[]);
    let zips = FakeServer::start(vec![]);
    let work = tempfile::tempdir().unwrap();
    let one = Zips::manifest("a", "one", Some("1.0.0"), 2);
    let two = Zips::manifest("a", "two", Some("1.0.0"), 2);
    let zip = Zips::raw(
        &work.path().join("two.zip"),
        &[("@a/one/amml.hcl", one.as_bytes()), ("@a/two/amml.hcl", two.as_bytes())],
    );
    zips.replace("/two.zip", Route::new(std::fs::read(zip).unwrap()));
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue =
        Catalogues::catalogue(vec![Catalogues::client(&registry, false, cache.path())], downloads.path());
    let result = UrlInspector::inspect(&mut catalogue, &zips.url("/two.zip"), &mut |_| {});
    assert!(matches!(result, Err(LauncherError::BadArchive { .. })));
    assert_eq!(std::fs::read_dir(downloads.path()).unwrap().count(), 0);
}

#[test]
fn a_registry_url_with_other_bytes_is_a_checksum_mismatch_and_not_kept() {
    let registry = FakeServer::start(vec![]);
    let zips = FakeServer::start(vec![]);
    let work = tempfile::tempdir().unwrap();
    let release = Catalogues::serve_mod(&zips, work.path(), "x", "y", "1.0.0", 2, &[]);
    let url = release.asset.clone();
    let expected = release.sha256.clone();
    RegistryTree::publish(&registry, "Reg", &amml(), &[RegistryTree::document("x", "y", false, true, vec![release])]);
    // the asset at the registry's URL is replaced
    let other = Zips::mod_zip(&work.path().join("other.zip"), "x", "y", "1.0.0", 2, &[("evil.dll", b"evil")]);
    zips.replace("/zips/x-y-1.0.0.zip", Route::new(std::fs::read(other).unwrap()));
    let cache = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let mut catalogue =
        Catalogues::catalogue(vec![Catalogues::client(&registry, false, cache.path())], downloads.path());

    match UrlInspector::inspect(&mut catalogue, &url, &mut |_| {}) {
        Err(LauncherError::ChecksumMismatch { expected: named, .. }) => assert_eq!(named, expected),
        other => panic!("{:?}", other.map(|_| ())),
    }
    assert_eq!(std::fs::read_dir(downloads.path()).unwrap().count(), 0);
}
