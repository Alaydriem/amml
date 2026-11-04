use crate::support::{FakeHttp, RegistryFixture};
use amml_launcher_bridge::{CardStateView, CatalogService, CatalogSortView};
use amml_launcher_core::{RegistryClient, RegistrySet};

const BASE: &str = "https://one.test/";

fn set_of(clients: Vec<RegistryClient<FakeHttp>>) -> RegistrySet<FakeHttp> {
    RegistrySet::new(clients)
}

fn id_of(client: &RegistryClient<FakeHttp>) -> String {
    client.registry().cache_key()
}

#[test]
fn a_page_marks_installed_and_updatable_cards() {
    let cache = tempfile::tempdir().unwrap();
    let documents = vec![
        RegistryFixture::document("ns", "alpha", "1.0.0", 10, "2026-10-01T00:00:00Z", 2),
        RegistryFixture::document("ns", "beta", "2.0.0", 20, "2026-10-02T00:00:00Z", 2),
        RegistryFixture::document("ns", "gamma", "3.0.0", 30, "2026-10-03T00:00:00Z", 2),
    ];
    let (client, _) = RegistryFixture::serve(BASE, &documents, cache.path());
    let id = id_of(&client);
    let mut set = set_of(vec![client]);
    let installed =
        [("@ns/alpha".to_string(), Some("1.0.0".to_string())), ("@ns/beta".to_string(), Some("1.5.0".to_string()))];

    let cards = CatalogService::page(&mut set, &id, 2, 1, &installed).unwrap();

    let states: Vec<(String, CardStateView)> = cards.iter().map(|card| (card.id.clone(), card.state)).collect();
    assert_eq!(
        states,
        vec![
            ("@ns/alpha".to_string(), CardStateView::Installed),
            ("@ns/beta".to_string(), CardStateView::Update),
            ("@ns/gamma".to_string(), CardStateView::Available),
        ]
    );
    assert_eq!(cards[0].registry_id, id);
}

#[test]
fn an_icon_url_is_the_registry_base_plus_the_card_path() {
    let cache = tempfile::tempdir().unwrap();
    let documents = vec![RegistryFixture::document("ns", "alpha", "1.0.0", 10, "2026-10-01T00:00:00Z", 2)];
    let (client, _) = RegistryFixture::serve(BASE, &documents, cache.path());
    let id = id_of(&client);
    let mut set = set_of(vec![client]);

    let cards = CatalogService::page(&mut set, &id, 2, 1, &[]).unwrap();

    assert_eq!(cards[0].icon_url.as_deref(), Some("https://one.test/icons/alpha.png"));
}

#[test]
fn the_layout_counts_groups_and_gives_the_page_size() {
    let cache = tempfile::tempdir().unwrap();
    let mut documents = RegistryFixture::many("a", 3);
    documents.push(RegistryFixture::document("zed", "last", "1.0.0", 5, "2026-10-01T00:00:00Z", 2));
    let (client, _) = RegistryFixture::serve(BASE, &documents, cache.path());
    let id = id_of(&client);
    let mut set = set_of(vec![client]);

    let layout = CatalogService::layout(&mut set, &id, 2).unwrap();

    assert_eq!(layout.page_size, 100);
    assert_eq!(layout.total, 4);
    let letters: Vec<(&str, u32, u32)> = layout.groups.iter().map(|g| (g.id.as_str(), g.start, g.total)).collect();
    assert_eq!(letters, vec![("N", 0, 3), ("Z", 3, 1)]);
}

#[test]
fn search_fetches_only_the_pages_that_hold_hits() {
    let cache = tempfile::tempdir().unwrap();
    let mut documents = RegistryFixture::many("filler", 100);
    documents.push(RegistryFixture::document("zed", "needle", "1.0.0", 5, "2026-10-01T00:00:00Z", 2));
    let (client, http) = RegistryFixture::serve(BASE, &documents, cache.path());
    let mut set = set_of(vec![client]);

    let groups = CatalogService::search(&mut set, "needle", 2, &[]);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].cards.len(), 1);
    assert_eq!(groups[0].cards[0].id, "@zed/needle");
    assert_eq!(http.requested("https://one.test/index/api-2/0002.json"), 1);
    assert_eq!(http.requested("https://one.test/index/api-2/0001.json"), 0);
}

#[test]
fn search_keeps_fifty_cards_per_registry_and_the_full_total() {
    let cache = tempfile::tempdir().unwrap();
    let documents = RegistryFixture::many("cam", 60);
    let (client, _) = RegistryFixture::serve(BASE, &documents, cache.path());
    let mut set = set_of(vec![client]);

    let groups = CatalogService::search(&mut set, "cam", 2, &[]);

    assert_eq!(groups[0].total, 60);
    assert_eq!(groups[0].cards.len(), CatalogService::SEARCH_LIMIT);
    assert_eq!(groups[0].cards[0].tags, vec!["camera".to_string()]);
}

#[test]
fn a_failing_registry_drops_out_of_search_and_the_others_answer() {
    let cache = tempfile::tempdir().unwrap();
    let documents = vec![RegistryFixture::document("ns", "camera", "1.0.0", 5, "2026-10-01T00:00:00Z", 2)];
    let (good, _) = RegistryFixture::serve(BASE, &documents, &cache.path().join("good"));
    let (broken, down) = RegistryFixture::serve("https://two.test/", &documents, &cache.path().join("broken"));
    down.go_offline();
    let good_id = id_of(&good);
    let mut set = set_of(vec![broken, good]);

    let groups = CatalogService::search(&mut set, "camera", 2, &[]);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].registry_id, good_id);
}

#[test]
fn search_ignores_mods_that_list_only_another_api() {
    let cache = tempfile::tempdir().unwrap();
    let documents = vec![
        RegistryFixture::document("ns", "camera", "1.0.0", 5, "2026-10-01T00:00:00Z", 1),
        RegistryFixture::document("ns", "camera-two", "1.0.0", 5, "2026-10-01T00:00:00Z", 2),
    ];
    let (client, _) = RegistryFixture::serve(BASE, &documents, cache.path());
    let mut set = set_of(vec![client]);

    let groups = CatalogService::search(&mut set, "camera", 2, &[]);

    assert_eq!(groups[0].total, 1);
    assert_eq!(groups[0].cards[0].id, "@ns/camera-two");
}

#[test]
fn sorting_by_size_needs_every_page_and_gives_none_past_ten_pages() {
    let cache = tempfile::tempdir().unwrap();
    let documents = RegistryFixture::many("m", 150);
    let (client, http) = RegistryFixture::serve(BASE, &documents, cache.path());
    let id = id_of(&client);
    let mut set = set_of(vec![client]);

    let by_size = CatalogService::sorted(&mut set, &id, 2, CatalogSortView::Size, &[]).unwrap().unwrap();

    assert_eq!(by_size.len(), 150);
    assert!(by_size.windows(2).all(|pair| pair[0].size >= pair[1].size));
    assert_eq!(http.requested("https://one.test/index/api-2/0001.json"), 1);
    assert_eq!(http.requested("https://one.test/index/api-2/0002.json"), 1);
    assert!(CatalogService::sorted(&mut set, &id, 2, CatalogSortView::Name, &[]).unwrap().is_none());

    let big_cache = tempfile::tempdir().unwrap();
    let (big, _) = RegistryFixture::serve("https://big.test/", &RegistryFixture::many("b", 1001), big_cache.path());
    let big_id = id_of(&big);
    let mut big_set = set_of(vec![big]);
    assert!(CatalogService::sorted(&mut big_set, &big_id, 2, CatalogSortView::Updated, &[]).unwrap().is_none());
}

#[test]
fn sorting_by_updated_puts_the_newest_first() {
    let cache = tempfile::tempdir().unwrap();
    let documents = vec![
        RegistryFixture::document("ns", "old", "1.0.0", 5, "2026-01-01T00:00:00Z", 2),
        RegistryFixture::document("ns", "new", "1.0.0", 5, "2026-10-01T00:00:00Z", 2),
        RegistryFixture::document("ns", "mid", "1.0.0", 5, "2026-05-01T00:00:00Z", 2),
    ];
    let (client, _) = RegistryFixture::serve(BASE, &documents, cache.path());
    let id = id_of(&client);
    let mut set = set_of(vec![client]);

    let cards = CatalogService::sorted(&mut set, &id, 2, CatalogSortView::Updated, &[]).unwrap().unwrap();

    let order: Vec<&str> = cards.iter().map(|card| card.id.as_str()).collect();
    assert_eq!(order, vec!["@ns/new", "@ns/mid", "@ns/old"]);
}

#[test]
fn a_detail_carries_the_document_fields_and_the_release_for_the_api() {
    let cache = tempfile::tempdir().unwrap();
    let documents = vec![RegistryFixture::document("ns", "alpha", "1.0.0", 10, "2026-10-01T00:00:00Z", 2)];
    let (client, _) = RegistryFixture::serve(BASE, &documents, cache.path());
    let id = id_of(&client);
    let mut set = set_of(vec![client]);

    let detail = CatalogService::detail(&mut set, &id, "@ns/alpha", 2).unwrap();

    assert_eq!(detail.card.version, "1.0.0");
    assert_eq!(detail.description.as_deref(), Some("alpha description"));
    assert_eq!(detail.sha256, "ab".repeat(32));
    assert!(!detail.stale);
    assert!(CatalogService::detail(&mut set, &id, "@ns/alpha", 9).is_err());
}
