use crate::support::RegistryTree;
use amml_gdk::GameVersion;
use amml_launcher_core::Compatibility;

#[test]
fn the_newest_amml_listing_the_game_wins() {
    let root = RegistryTree::root(RegistryTree::amml(vec![
        RegistryTree::amml_release("26.61.0", 3, &["1.26.61.1"]),
        RegistryTree::amml_release("26.60.1", 2, &["1.26.60.5"]),
        RegistryTree::amml_release("26.60.0", 2, &["1.26.60.5", "garbage"]),
    ]));
    let game = GameVersion::parse("1.26.60.05").unwrap();
    assert_eq!(Compatibility::amml_for(&game, &root).unwrap().version, "26.60.1");
    let newer = GameVersion::parse("1.26.61.1").unwrap();
    assert_eq!(Compatibility::amml_for(&newer, &root).unwrap().version, "26.61.0");
}

#[test]
fn versions_compare_by_number_not_text() {
    let root = RegistryTree::root(RegistryTree::amml(vec![
        RegistryTree::amml_release("26.60.9", 2, &["1.26.60.5"]),
        RegistryTree::amml_release("26.60.10", 2, &["1.26.60.5"]),
    ]));
    let game = GameVersion::parse("1.26.60.5").unwrap();
    assert_eq!(Compatibility::amml_for(&game, &root).unwrap().version, "26.60.10");
}

#[test]
fn no_amml_for_the_game_is_none() {
    let root = RegistryTree::root(RegistryTree::amml(vec![RegistryTree::amml_release("26.60.1", 2, &["1.26.60.5"])]));
    assert!(Compatibility::amml_for(&GameVersion::parse("1.26.70.1").unwrap(), &root).is_none());
}

#[test]
fn newest_for_api_ignores_other_apis() {
    let doc = RegistryTree::document(
        "alaydriem",
        "zoom",
        false,
        true,
        vec![RegistryTree::release("26.60.2", 3, "u", "h", 1), RegistryTree::release("26.60.1", 2, "u", "h", 1)],
    );
    assert_eq!(Compatibility::newest_for_api(&doc, 2).unwrap().version, "26.60.1");
    assert_eq!(Compatibility::newest_for_api(&doc, 3).unwrap().version, "26.60.2");
    assert!(Compatibility::newest_for_api(&doc, 4).is_none());
}

#[test]
fn installable_games_are_the_union_newest_first() {
    let root = RegistryTree::root(RegistryTree::amml(vec![
        RegistryTree::amml_release("26.61.0", 3, &["1.26.61.1", "1.26.60.5"]),
        RegistryTree::amml_release("26.60.1", 2, &["1.26.60.5", "1.26.60.4"]),
    ]));
    let games: Vec<String> = Compatibility::installable_games(&root).iter().map(|game| game.to_string()).collect();
    assert_eq!(games, vec!["1.26.61.1", "1.26.60.5", "1.26.60.4"]);
}
