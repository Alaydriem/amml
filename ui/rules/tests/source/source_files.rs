use amml_ui_rules::source::SourceFiles;

use crate::support::SourceTree;

#[test]
fn finds_nested_files_and_mod_rs() {
    let tree = SourceTree::new();
    let lib = tree.file("lib.rs", "");
    let module = tree.file("widget/mod.rs", "");
    let nested = tree.file("widget/button/text_button.rs", "");

    let files = SourceFiles::new(tree.root()).rust_files().unwrap_or_else(|error| panic!("{error}"));

    let mut expected = vec![lib, nested, module];
    expected.sort();
    assert_eq!(files, expected);
}

#[test]
fn skips_an_exempt_folder_at_any_depth() {
    let tree = SourceTree::new();
    let kept = tree.file("widget/chip.rs", "");
    tree.file("tokens/color/accent.rs", "");
    tree.file("theme/tokens/rule/soft_fill.rs", "");

    let files =
        SourceFiles::new(tree.root()).exempt_dir("tokens").rust_files().unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(files, vec![kept]);
}

#[test]
fn ignores_files_that_are_not_rust() {
    let tree = SourceTree::new();
    let kept = tree.file("lib.rs", "");
    tree.file("notes.md", "");
    tree.file("assets/icon.png", "");
    tree.file("lib.rs.bak", "");

    let files = SourceFiles::new(tree.root()).rust_files().unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(files, vec![kept]);
}

#[test]
fn a_missing_root_is_an_error() {
    let tree = SourceTree::new();
    assert!(SourceFiles::new(tree.root().join("absent")).rust_files().is_err());
}
