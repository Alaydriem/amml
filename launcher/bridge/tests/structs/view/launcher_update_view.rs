use amml_launcher_bridge::LauncherUpdateView;

#[test]
fn the_page_is_the_release_tag_of_the_version() {
    let view = LauncherUpdateView::new("26.63.5", true);

    assert_eq!(view.version, "26.63.5");
    assert!(view.installs);
    assert_eq!(view.page, "https://github.com/alaydriem/amml/releases/tag/26.63.5");
}
