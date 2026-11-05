use crate::support::RegistryTree;
use amml_gdk::GameVersion;
use amml_launcher_core::{AmmlLevel, AmmlState, Compatibility};
use amml_registry::ReleaseVersion;

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

fn state(version: Option<&str>, local: bool) -> AmmlState {
    AmmlState { version: version.map(str::to_string), installed: true, local, local_api: None }
}

fn release_level(text: &str) -> AmmlLevel {
    AmmlLevel::Release(ReleaseVersion::parse(text).unwrap())
}

#[test]
fn level_of_local_build_is_local() {
    assert_eq!(AmmlLevel::of(&state(Some("26.60.5+local"), true)), AmmlLevel::Local);
}

#[test]
fn level_of_missing_version_is_unknown() {
    assert_eq!(AmmlLevel::of(&state(None, false)), AmmlLevel::Unknown);
    assert_eq!(AmmlLevel::of(&state(Some("garbage"), false)), AmmlLevel::Unknown);
}

#[test]
fn level_of_plain_version_is_release() {
    assert_eq!(AmmlLevel::of(&state(Some("26.60.5"), false)), release_level("26.60.5"));
}

#[test]
fn fits_without_min_always() {
    let release = RegistryTree::release_min("1.0.0", 2, None);
    assert!(Compatibility::fits(&release, &AmmlLevel::Unknown));
    assert!(Compatibility::fits(&release, &release_level("26.60.5")));
    assert!(Compatibility::fits(&release, &AmmlLevel::Local));
}

#[test]
fn fits_local_always() {
    let release = RegistryTree::release_min("1.0.0", 2, Some("99.0.0"));
    assert!(Compatibility::fits(&release, &AmmlLevel::Local));
}

#[test]
fn unknown_level_fits_only_releases_without_min() {
    assert!(!Compatibility::fits(&RegistryTree::release_min("1.0.0", 2, Some("26.60.0")), &AmmlLevel::Unknown));
    assert!(Compatibility::fits(&RegistryTree::release_min("1.0.0", 2, None), &AmmlLevel::Unknown));
}

#[test]
fn fits_compares_parsed_versions() {
    let level = release_level("26.60.10");
    assert!(Compatibility::fits(&RegistryTree::release_min("1.0.0", 2, Some("26.60.9")), &level));
    assert!(Compatibility::fits(&RegistryTree::release_min("1.0.0", 2, Some("26.060.10")), &level));
    assert!(!Compatibility::fits(&RegistryTree::release_min("1.0.0", 2, Some("26.61.0")), &level));
    assert!(!Compatibility::fits(&RegistryTree::release_min("1.0.0", 2, Some("junk")), &level));
}

fn doc(releases: Vec<amml_registry::ModRelease>) -> amml_registry::ModDocument {
    RegistryTree::document("alaydriem", "zoom", false, true, releases)
}

#[test]
fn choose_newest_fits_has_no_blocked() {
    let document = doc(vec![
        RegistryTree::release_min("26.60.4", 2, None),
        RegistryTree::release_min("26.60.5", 2, Some("26.60.5")),
    ]);
    let choice = Compatibility::choose(&document, 2, &release_level("26.60.5"), None);
    assert_eq!(choice.fitting.unwrap().version, "26.60.5");
    assert!(choice.blocked.is_none());
}

#[test]
fn choose_falls_back_to_older_fitting_and_reports_blocked() {
    let root = RegistryTree::root(RegistryTree::amml(vec![
        RegistryTree::amml_release("26.62.0", 3, &["1.26.61.1", "1.26.62.1", "garbage"]),
        RegistryTree::amml_release("26.60.5", 2, &["1.26.60.5"]),
    ]));
    let document = doc(vec![
        RegistryTree::release_min("26.60.4", 2, None),
        RegistryTree::release_min("26.62.0", 2, Some("26.62.0")),
    ]);
    let choice = Compatibility::choose(&document, 2, &release_level("26.60.5"), Some(&root));
    assert_eq!(choice.fitting.unwrap().version, "26.60.4");
    let blocked = choice.blocked.unwrap();
    assert_eq!(blocked.version, "26.62.0");
    assert_eq!(blocked.game, Some("1.26.62.1".to_string()));
}

#[test]
fn choose_nothing_fits() {
    let document = doc(vec![RegistryTree::release_min("26.62.0", 2, Some("26.62.0"))]);
    let choice = Compatibility::choose(&document, 2, &release_level("26.60.5"), None);
    assert!(choice.fitting.is_none());
    assert_eq!(choice.blocked.unwrap().version, "26.62.0");
}

#[test]
fn choose_blocked_game_none_when_amml_not_in_root() {
    let root = RegistryTree::root(RegistryTree::amml(vec![RegistryTree::amml_release("26.60.5", 2, &["1.26.60.5"])]));
    let document = doc(vec![RegistryTree::release_min("26.62.0", 2, Some("26.62.0"))]);
    let choice = Compatibility::choose(&document, 2, &AmmlLevel::Unknown, Some(&root));
    assert_eq!(choice.blocked.unwrap().game, None);
}

#[test]
fn choose_on_document_without_min_amml_matches_newest_for_api() {
    let document = doc(vec![
        RegistryTree::release_min("26.60.2", 3, None),
        RegistryTree::release_min("26.60.1", 2, None),
        RegistryTree::release_min("26.60.0", 2, None),
    ]);
    let choice = Compatibility::choose(&document, 2, &AmmlLevel::Unknown, None);
    assert_eq!(choice.fitting.unwrap().version, Compatibility::newest_for_api(&document, 2).unwrap().version);
    assert!(choice.blocked.is_none());
    let none = Compatibility::choose(&document, 4, &AmmlLevel::Unknown, None);
    assert!(none.fitting.is_none() && none.blocked.is_none());
}
