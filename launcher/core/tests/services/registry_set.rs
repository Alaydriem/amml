use crate::support::{FakeServer, RegistryTree, Route};
use amml_launcher_core::{
    CacheLock, RegistryCache, RegistryClient, RegistryRef, RegistrySet, RegistryStatus, UreqHttpFetch,
};

fn client(server: &FakeServer, cache: &std::path::Path) -> RegistryClient<UreqHttpFetch> {
    RegistryClient::new(
        RegistryRef::new(server.base()),
        UreqHttpFetch::new(UreqHttpFetch::agent()),
        RegistryCache::new(cache.to_path_buf(), RegistryCache::BUDGET, CacheLock::new()),
    )
}

fn serve_with(server: &FakeServer, names: &[&str]) {
    let docs: Vec<_> = names
        .iter()
        .map(|name| {
            RegistryTree::document(
                "alaydriem",
                name,
                false,
                true,
                vec![RegistryTree::release("26.60.1", 2, "u", "h", 1)],
            )
        })
        .collect();
    RegistryTree::publish(server, "Reg", &RegistryTree::amml(vec![]), &docs);
}

#[test]
fn search_groups_hits_by_registry_in_settings_order_and_skips_a_failing_one() {
    let first = FakeServer::start(vec![]);
    serve_with(&first, &["zoom"]);
    let failing = FakeServer::start(vec![]);
    serve_with(&failing, &["zoomer"]);
    failing.replace("/names.json", Route::new(b"{ not json".to_vec()));
    let last = FakeServer::start(vec![]);
    serve_with(&last, &["zoomy", "other"]);

    let cache = tempfile::tempdir().unwrap();
    let mut set = RegistrySet::new(vec![
        client(&first, &cache.path().join("1")),
        client(&failing, &cache.path().join("2")),
        client(&last, &cache.path().join("3")),
    ]);
    let found = set.search("zoom");
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].0.base, first.base());
    assert_eq!(found[0].1[0].id, "@alaydriem/zoom");
    assert_eq!(found[1].0.base, last.base());
    assert_eq!(found[1].1.len(), 1);
}

#[test]
fn refresh_all_reports_each_registry_and_the_official_one_is_found() {
    let up = FakeServer::start(vec![]);
    serve_with(&up, &["zoom"]);
    let down = FakeServer::start(vec![]);
    let down_base = down.base();
    drop(down);

    let cache = tempfile::tempdir().unwrap();
    let official = RegistryClient::new(
        RegistryRef::official_at(up.base()),
        UreqHttpFetch::new(UreqHttpFetch::agent()),
        RegistryCache::new(cache.path().join("1"), RegistryCache::BUDGET, CacheLock::new()),
    );
    let other = RegistryClient::new(
        RegistryRef::new(down_base.clone()),
        UreqHttpFetch::new(UreqHttpFetch::agent()),
        RegistryCache::new(cache.path().join("2"), RegistryCache::BUDGET, CacheLock::new()),
    );
    let mut set = RegistrySet::new(vec![other, official]);
    let statuses = set.refresh_all();
    assert_eq!(statuses[0].0.base, down_base);
    assert_eq!(statuses[0].1, RegistryStatus::Offline);
    assert_eq!(statuses[1].1, RegistryStatus::Ok);
    assert_eq!(set.official_mut().unwrap().registry().base, up.base());
}
