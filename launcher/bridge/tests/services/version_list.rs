use crate::support::Samples;
use amml_gdk::{Catalogue, CatalogueEntry, Channel, GameVersion, VersionFolder};
use amml_launcher_bridge::{ChannelView, VersionList, VersionStateView};
use std::path::PathBuf;

fn entry(version: &str, channel: Channel) -> CatalogueEntry {
    CatalogueEntry { version: GameVersion::parse(version).unwrap(), channel, urls: vec!["https://x/a.msixvc".into()] }
}

fn folder(version: &str, bytes: u64) -> VersionFolder {
    VersionFolder { version: GameVersion::parse(version).unwrap(), path: PathBuf::from("versions"), files: 3, bytes }
}

#[test]
fn a_version_no_amml_release_lists_is_unsupported() {
    let catalogue =
        Catalogue::new(vec![entry("1.26.60.5", Channel::Release), entry("1.21.0.3", Channel::Release)], vec![]);
    let amml = [Samples::amml_release("26.60.1", &["1.26.60.5"])];

    let views = VersionList::build(&catalogue, &[], &amml, &[], 4);

    assert_eq!(views[0].version, "1.26.60.05");
    assert_eq!(views[0].state, VersionStateView::Download);
    assert_eq!(views[0].amml.as_deref(), Some("26.60.1"));
    assert_eq!(views[0].default_mods, 4);
    assert_eq!(views[1].version, "1.21.0.03");
    assert_eq!(views[1].state, VersionStateView::Unsupported);
    assert_eq!(views[1].amml, None);
    assert_eq!(views[1].default_mods, 0);
}

#[test]
fn a_version_in_the_store_is_installed_with_its_size() {
    let catalogue = Catalogue::new(vec![entry("1.26.60.5", Channel::Release)], vec![]);
    let amml = [Samples::amml_release("26.60.1", &["1.26.60.5"])];

    let views = VersionList::build(&catalogue, &[folder("1.26.60.5", 9_000)], &amml, &[], 0);

    assert_eq!(views.len(), 1);
    assert_eq!(views[0].state, VersionStateView::Installed);
    assert_eq!(views[0].size, Some(9_000));
}

#[test]
fn version_keys_without_leading_zeros_match_instance_versions() {
    let dir = tempfile::tempdir().unwrap();
    let catalogue = Catalogue::new(vec![entry("1.26.60.5", Channel::Release)], vec![]);
    let amml = [Samples::amml_release("26.60.1", &["1.26.60.05"])];
    let mut instance = Samples::instance(dir.path(), "play");
    instance.game = GameVersion::parse("1.26.60.05").unwrap();

    let views = VersionList::build(&catalogue, &[], &amml, &[instance], 0);

    assert_eq!(views[0].instance_count, 1);
    assert_eq!(views[0].state, VersionStateView::Download);
}

#[test]
fn a_store_version_the_catalogue_lacks_is_listed_and_the_newest_comes_first() {
    let catalogue =
        Catalogue::new(vec![entry("1.26.52.3", Channel::Release)], vec![entry("1.26.70.20", Channel::Preview)]);

    let amml = [Samples::amml_release("26.60.1", &["1.26.60.5"])];

    let views = VersionList::build(&catalogue, &[folder("1.26.60.5", 1)], &amml, &[], 0);

    let order: Vec<&str> = views.iter().map(|view| view.version.as_str()).collect();
    assert_eq!(order, vec!["1.26.70.20", "1.26.60.05", "1.26.52.03"]);
    assert_eq!(views[0].channel, ChannelView::Preview);
    assert_eq!(views[1].channel, ChannelView::Release);
    assert_eq!(views[1].state, VersionStateView::Installed);
}

#[test]
fn the_newest_amml_release_that_lists_the_game_is_named() {
    let catalogue = Catalogue::new(vec![entry("1.26.60.5", Channel::Release)], vec![]);
    let amml = [
        Samples::amml_release("26.60.2", &["1.26.60.5"]),
        Samples::amml_release("26.60.10", &["1.26.60.5"]),
        Samples::amml_release("26.51.0", &["1.26.51.1"]),
    ];

    let views = VersionList::build(&catalogue, &[], &amml, &[], 0);

    assert_eq!(views[0].amml.as_deref(), Some("26.60.10"));
}

#[test]
fn a_version_in_the_store_that_no_amml_release_lists_is_not_supported_for_a_new_instance() {
    let catalogue = Catalogue::new(vec![entry("1.26.60.5", Channel::Release)], vec![]);
    let amml = [Samples::amml_release("26.60.1", &["1.26.52.3"])];

    let views = VersionList::build(&catalogue, &[folder("1.26.60.5", 9_000)], &amml, &[], 4);

    assert_eq!(views[0].state, VersionStateView::Unsupported);
    assert_eq!(views[0].amml, None);
    assert_eq!(views[0].default_mods, 0);

    let none_at_all = VersionList::build(&catalogue, &[folder("1.26.60.5", 9_000)], &[], &[], 0);
    assert_eq!(none_at_all[0].state, VersionStateView::Unsupported);
}

#[test]
fn the_settings_page_still_lists_a_version_on_disk_that_no_amml_release_lists() {
    let dir = tempfile::tempdir().unwrap();
    let mut preview = Samples::instance(dir.path(), "beta");
    preview.game = GameVersion::parse("1.26.70.20").unwrap();
    preview.preview = true;
    let mut release = Samples::instance(dir.path(), "play");
    release.game = GameVersion::parse("1.26.60.5").unwrap();

    let views = VersionList::on_disk(&[folder("1.26.60.5", 9_000), folder("1.26.70.20", 5_000)], &[release, preview]);

    let shown: Vec<(&str, VersionStateView, Option<u64>, u32)> =
        views.iter().map(|view| (view.version.as_str(), view.state, view.size, view.instance_count)).collect();
    assert_eq!(
        shown,
        vec![
            ("1.26.70.20", VersionStateView::Installed, Some(5_000), 1),
            ("1.26.60.05", VersionStateView::Installed, Some(9_000), 1)
        ]
    );
    assert_eq!(views[0].channel, ChannelView::Preview);
    assert_eq!(views[1].channel, ChannelView::Release);
}
