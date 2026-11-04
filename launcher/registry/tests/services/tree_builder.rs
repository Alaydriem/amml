use amml_registry::{AmmlRelease, AmmlSection};
use amml_registry::{IndexPage, ModDocument, NamesFile, PublishedTree, RegistryRoot, TreeBuilder};
use sha2::{Digest, Sha256};

use crate::support::Samples;

fn amml() -> AmmlSection {
    AmmlSection {
        repo: "alaydriem/amml".to_string(),
        versions: vec![AmmlRelease {
            version: "26.60.1".to_string(),
            api_version: 2,
            game_versions: vec!["1.26.60.5".to_string()],
            asset: "https://example.test/amml.zip".to_string(),
            sha256: "ab".to_string(),
            size: 5,
        }],
    }
}

fn simple(name: &str, api: u32) -> ModDocument {
    Samples::document("alaydriem", name, vec![Samples::release("26.60.1", api, "aa", Samples::NOW)])
}

fn build(docs: &[ModDocument]) -> PublishedTree {
    TreeBuilder::build("Official", Samples::NOW, &amml(), docs).unwrap()
}

fn page(tree: &PublishedTree, path: &str) -> IndexPage {
    serde_json::from_slice(&tree.files[path]).unwrap()
}

#[test]
fn output_does_not_depend_on_input_order() {
    let docs: Vec<_> = ["zoom", "Alpha", "minimap", "beta"].iter().map(|name| simple(name, 2)).collect();
    let mut reversed = docs.clone();
    reversed.reverse();
    assert_eq!(build(&docs), build(&reversed));
    assert_eq!(build(&docs), build(&docs));
}

#[test]
fn pages_hold_one_hundred_cards_sorted_without_case() {
    let docs: Vec<_> = (0..250).map(|n| simple(&format!("mod{n:03}"), 2)).collect();
    let tree = build(&docs);
    let first = page(&tree, "index/api-2/0001.json");
    let second = page(&tree, "index/api-2/0002.json");
    let third = page(&tree, "index/api-2/0003.json");
    assert_eq!((first.cards.len(), second.cards.len(), third.cards.len()), (100, 100, 50));
    assert_eq!(first.cards[0].id, "@alaydriem/mod000");
    assert_eq!(second.cards[0].id, "@alaydriem/mod100");
    assert_eq!(third.page, 3);
    assert!(!tree.files.contains_key("index/api-2/0004.json"));

    let root: RegistryRoot = serde_json::from_slice(&tree.files["registry.json"]).unwrap();
    assert_eq!(root.mod_count, 250);
    assert_eq!(root.index.page_size, 100);
    assert_eq!(root.index.apis["2"].mods, 250);
    assert_eq!(root.index.apis["2"].pages, 3);
}

#[test]
fn sorting_ignores_case() {
    let docs = vec![simple("beta", 2), simple("Alpha", 2), simple("zoom", 2)];
    let ids: Vec<_> = page(&build(&docs), "index/api-2/0001.json").cards.into_iter().map(|card| card.id).collect();
    assert_eq!(ids, ["@alaydriem/Alpha", "@alaydriem/beta", "@alaydriem/zoom"]);
}

#[test]
fn a_mod_is_listed_under_every_api_it_has_a_version_for() {
    let both = Samples::document(
        "alaydriem",
        "zoom",
        vec![
            Samples::release("26.60.2", 2, "bb", "2026-11-02T00:00:00Z"),
            Samples::release("26.51.9", 1, "aa", "2026-10-01T00:00:00Z"),
            Samples::release("26.51.3", 1, "cc", "2026-09-01T00:00:00Z"),
        ],
    );
    let tree = build(&[both, simple("old", 1)]);
    let api2 = page(&tree, "index/api-2/0001.json");
    let api1 = page(&tree, "index/api-1/0001.json");
    assert_eq!(api2.cards.len(), 1);
    assert_eq!(api1.cards.len(), 2);
    assert_eq!(api2.cards[0].version, "26.60.2");
    assert_eq!(api2.cards[0].updated, "2026-11-02T00:00:00Z");
    let zoom_api1 = api1.cards.iter().find(|card| card.id == "@alaydriem/zoom").unwrap();
    assert_eq!(zoom_api1.version, "26.51.9");

    let names: NamesFile = serde_json::from_slice(&tree.files["names.json"]).unwrap();
    let zoom = names.rows.iter().find(|row| row.0 == "@alaydriem/zoom").unwrap();
    assert_eq!(zoom.3, [2, 1]);
    assert_eq!(zoom.4, 1);
    let old = names.rows.iter().find(|row| row.0 == "@alaydriem/old").unwrap();
    assert_eq!(old.3, [1]);
}

#[test]
fn names_row_page_is_the_page_in_the_newest_api() {
    let docs: Vec<_> = (0..150).map(|n| simple(&format!("mod{n:03}"), 2)).collect();
    let names: NamesFile = serde_json::from_slice(&build(&docs).files["names.json"]).unwrap();
    assert_eq!(names.rows[0].4, 1);
    assert_eq!(names.rows[149].4, 2);
}

#[test]
fn root_describes_names_file_by_hash_and_size() {
    let tree = build(&[simple("zoom", 2)]);
    let root: RegistryRoot = serde_json::from_slice(&tree.files["registry.json"]).unwrap();
    let names = &tree.files["names.json"];
    assert_eq!(root.names.path, "names.json");
    assert_eq!(root.names.size, names.len() as u64);
    let digest = Sha256::digest(names);
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(root.names.sha256, hex);
    assert_eq!(root.amml.versions[0].version, "26.60.1");
    assert_eq!(root.published, Samples::NOW);
    assert_eq!(root.schema, 2);
}

#[test]
fn files_are_small_enough_for_the_launcher_limits() {
    let docs: Vec<_> = (0..100).map(|n| simple(&format!("mod{n:03}"), 2)).collect();
    let tree = build(&docs);
    assert!(tree.files["registry.json"].len() < 64 * 1024);
    assert!(tree.files["index/api-2/0001.json"].len() < 256 * 1024);
}

#[test]
fn an_empty_registry_has_a_root_and_an_empty_names_file() {
    let tree = build(&[]);
    assert!(tree.files.contains_key("registry.json"));
    let names: NamesFile = serde_json::from_slice(&tree.files["names.json"]).unwrap();
    assert!(names.rows.is_empty());
    assert!(!tree.files.keys().any(|path| path.starts_with("index/")));
}
