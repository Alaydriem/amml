use crate::support::{FakeServer, RegistryTree, Route};
use amml_launcher_core::{
    LauncherError, ModId, RegistryCache, RegistryClient, RegistryRef, RegistryStatus, UreqHttpFetch,
};
use amml_registry::{IndexCard, IndexPage, JsonText};
use std::path::Path;

fn client(server_base: &str, cache_dir: &Path) -> RegistryClient<UreqHttpFetch> {
    RegistryClient::new(
        RegistryRef::new(server_base.to_string()),
        UreqHttpFetch::new(UreqHttpFetch::agent()),
        RegistryCache::new(cache_dir.to_path_buf(), RegistryCache::BUDGET),
    )
}

fn zoom_docs() -> Vec<amml_registry::ModDocument> {
    vec![RegistryTree::document(
        "alaydriem",
        "zoom",
        false,
        true,
        vec![RegistryTree::release("26.60.1", 2, "https://example.test/zoom.zip", "aa", 10)],
    )]
}

fn amml() -> amml_registry::AmmlSection {
    RegistryTree::amml(vec![RegistryTree::amml_release("26.60.1", 2, &["1.26.60.5"])])
}

fn page_json(api: u32, page: u32) -> Vec<u8> {
    let card = IndexCard {
        id: "@a/b".to_string(),
        title: "B".to_string(),
        summary: "s".to_string(),
        version: "26.60.1".to_string(),
        size: 1,
        icon: None,
        core: false,
        default: false,
        updated: RegistryTree::NOW.to_string(),
    };
    JsonText::compact(&IndexPage { schema: 2, api_version: api, page, cards: vec![card] }).unwrap()
}

#[test]
fn a_fresh_root_is_ok_and_a_repeat_sends_the_etag() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Official", &amml(), &zoom_docs());
    let cache = tempfile::tempdir().unwrap();
    let mut client = client(&server.base(), cache.path());
    assert_eq!(client.refresh(), RegistryStatus::Ok);
    assert_eq!(client.name(), "Official");
    assert_eq!(client.root().unwrap().mod_count, 1);

    assert_eq!(client.refresh(), RegistryStatus::Ok);
    let roots: Vec<_> = server.requests().into_iter().filter(|request| request.path == "/registry.json").collect();
    assert_eq!(roots.len(), 2);
    assert_eq!(roots[0].if_none_match, None);
    assert!(roots[1].if_none_match.is_some());
}

#[test]
fn schema_1_or_an_oversized_root_is_broken() {
    let server = FakeServer::start(vec![("/registry.json", Route::new(b"{\"schema\":1}".to_vec()))]);
    let cache = tempfile::tempdir().unwrap();
    let mut first = client(&server.base(), cache.path());
    assert!(matches!(first.refresh(), RegistryStatus::Broken { .. }));
    assert!(first.root().is_none());
    assert_eq!(first.name(), server.base());

    server.replace("/registry.json", Route::new(vec![b' '; 70 * 1024]));
    let other_cache = tempfile::tempdir().unwrap();
    let mut second = client(&server.base(), other_cache.path());
    match second.refresh() {
        RegistryStatus::Broken { reason } => assert!(reason.contains("limit"), "{reason}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn offline_with_a_cached_root_is_stale_and_without_one_is_offline() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Official", &amml(), &zoom_docs());
    let base = server.base();
    let cache = tempfile::tempdir().unwrap();
    assert_eq!(client(&base, cache.path()).refresh(), RegistryStatus::Ok);
    drop(server);

    let mut stale = client(&base, cache.path());
    assert!(matches!(stale.refresh(), RegistryStatus::Stale { .. }));
    assert_eq!(stale.root().unwrap().name, "Official");

    let empty = tempfile::tempdir().unwrap();
    let mut none = client(&base, empty.path());
    assert_eq!(none.refresh(), RegistryStatus::Offline);
    assert!(none.root().is_none());
}

#[test]
fn pages_beyond_ten_are_dropped_from_memory_least_recent_first() {
    let server = FakeServer::start(vec![]);
    for page in 1..=11 {
        let route = Route::with_etag(page_json(2, page), &format!("\"p{page}\""));
        server.replace(&format!("/index/api-2/{page:04}.json"), route);
    }
    let cache = tempfile::tempdir().unwrap();
    let mut client = client(&server.base(), cache.path());
    for page in 1..=11 {
        assert_eq!(client.page(2, page).unwrap().value.page, page);
    }
    assert_eq!(client.pages_in_memory(), 10);

    server.clear_requests();
    client.page(2, 11).unwrap();
    assert!(server.requests().is_empty());

    client.page(2, 1).unwrap();
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/index/api-2/0001.json");
    assert_eq!(requests[0].if_none_match.as_deref(), Some("\"p1\""));
    assert_eq!(client.pages_in_memory(), 10);
}

#[test]
fn a_page_for_another_api_or_number_is_refused() {
    let server = FakeServer::start(vec![("/index/api-2/0001.json", Route::new(page_json(3, 1)))]);
    let cache = tempfile::tempdir().unwrap();
    let mut client = client(&server.base(), cache.path());
    assert!(matches!(client.page(2, 1), Err(LauncherError::Http { .. })));
}

#[test]
fn a_mod_document_offline_comes_from_the_cache_marked_stale() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Official", &amml(), &zoom_docs());
    let base = server.base();
    let cache = tempfile::tempdir().unwrap();
    let id = ModId::parse("@alaydriem/zoom").unwrap();
    let online = client(&base, cache.path()).document(&id).unwrap();
    assert!(!online.stale);
    drop(server);

    let offline = client(&base, cache.path()).document(&id).unwrap();
    assert!(offline.stale);
    assert_eq!(offline.value.id, "@alaydriem/zoom");

    let empty = tempfile::tempdir().unwrap();
    assert!(matches!(client(&base, empty.path()).document(&id), Err(LauncherError::Offline { .. })));
}

#[test]
fn a_document_for_another_mod_is_refused() {
    let server = FakeServer::start(vec![]);
    let tree = RegistryTree::build("Official", &amml(), &zoom_docs());
    let wrong = tree.files["mods/@alaydriem/zoom.json"].clone();
    server.replace("/mods/@alaydriem/peek.json", Route::new(wrong));
    let cache = tempfile::tempdir().unwrap();
    let id = ModId::parse("@alaydriem/peek").unwrap();
    assert!(matches!(client(&server.base(), cache.path()).document(&id), Err(LauncherError::Http { .. })));
}

#[test]
fn an_icon_path_outside_icons_or_over_64_kb_is_refused() {
    let server = FakeServer::start(vec![
        ("/icons/@a/ok.png", Route::new(vec![1u8; 100])),
        ("/icons/@a/big.png", Route::new(vec![1u8; 70 * 1024])),
        ("/other/x.png", Route::new(vec![1u8; 10])),
    ]);
    let cache = tempfile::tempdir().unwrap();
    let mut client = client(&server.base(), cache.path());
    assert_eq!(client.icon("icons/@a/ok.png").unwrap().value.len(), 100);
    assert!(matches!(client.icon("other/x.png"), Err(LauncherError::BadModId(_))));
    assert!(matches!(client.icon("icons/../x.png"), Err(LauncherError::BadModId(_))));
    assert!(matches!(client.icon("icons/@a/big.png"), Err(LauncherError::TooLarge { .. })));
}

#[test]
fn names_that_do_not_match_the_root_hash_are_refused() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Official", &amml(), &zoom_docs());
    server.replace("/names.json", Route::new(b"{\"schema\":2,\"rows\":[]}\n".to_vec()));
    let cache = tempfile::tempdir().unwrap();
    let mut client = client(&server.base(), cache.path());
    assert!(matches!(client.names(), Err(LauncherError::ChecksumMismatch { .. })));
}

#[test]
fn names_are_fetched_once_per_client() {
    let server = FakeServer::start(vec![]);
    RegistryTree::publish(&server, "Official", &amml(), &zoom_docs());
    let cache = tempfile::tempdir().unwrap();
    let mut client = client(&server.base(), cache.path());
    assert_eq!(client.names().unwrap().value.rows.len(), 1);
    client.names().unwrap();
    assert_eq!(server.count("/names.json"), 1);
}
