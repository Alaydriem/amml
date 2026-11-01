use amml_ui_rules::folder::ForbiddenFolders;

use crate::support::SourceTree;

#[test]
fn a_widgets_folder_is_present() {
    let tree = SourceTree::new();
    tree.file("widgets/bar_button.rs", "");

    let present = ForbiddenFolders::new(tree.root(), &["widgets", "theme"]).present();

    assert_eq!(present, vec![tree.root().join("widgets")]);
}

#[test]
fn an_empty_widgets_folder_is_present() {
    let tree = SourceTree::new();
    tree.folder("theme");

    let present = ForbiddenFolders::new(tree.root(), &["widgets", "theme"]).present();

    assert_eq!(present, vec![tree.root().join("theme")]);
}

#[test]
fn a_theme_rs_file_is_present() {
    let tree = SourceTree::new();
    tree.file("theme.rs", "");

    let present = ForbiddenFolders::new(tree.root(), &["widgets", "theme"]).present();

    assert_eq!(present, vec![tree.root().join("theme.rs")]);
}

#[test]
fn a_clean_root_has_none() {
    let tree = SourceTree::new();
    tree.file("views/bar.rs", "");
    tree.file("views/widgets_row.rs", "");

    assert!(ForbiddenFolders::new(tree.root(), &["widgets", "theme"]).present().is_empty());
}
