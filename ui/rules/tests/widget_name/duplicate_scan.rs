use std::path::Path;

use amml_ui_rules::source::SourceFiles;
use amml_ui_rules::widget_name::{DuplicateScan, WidgetNames};

use crate::support::SourceTree;

/// The amml-ui names `TextButton` and `Toast`, read from a planted widget folder.
struct PlantedNames;

impl PlantedNames {
    fn names() -> (SourceTree, WidgetNames) {
        let tree = SourceTree::new();
        tree.file("widget/text_button.rs", "pub struct TextButton;\n");
        tree.file("widget/toast.rs", "pub struct Toast;\n");
        let names = WidgetNames::from_source(&SourceFiles::new(tree.root().join("widget")))
            .unwrap_or_else(|error| panic!("{error}"));
        (tree, names)
    }
}

#[test]
fn a_private_struct_with_a_widget_name_is_a_hit() {
    let (_tree, names) = PlantedNames::names();
    let text = "use super::Notice;\n\nstruct Toast {\n    text: Notice,\n}\n";

    let hits = DuplicateScan::scan_text(&names, Path::new("state/toast.rs"), text);

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].name, "Toast");
    assert_eq!(hits[0].line, 3);
    assert_eq!(hits[0].to_string(), "state/toast.rs:3: Toast is an amml-ui widget name");
}

#[test]
fn an_enum_or_public_struct_with_a_widget_name_is_a_hit() {
    let (_tree, names) = PlantedNames::names();

    assert_eq!(DuplicateScan::scan_text(&names, Path::new("a.rs"), "pub enum Toast {").len(), 1);
    assert_eq!(DuplicateScan::scan_text(&names, Path::new("a.rs"), "pub(crate) struct TextButton<'a> {").len(), 1);
}

#[test]
fn a_longer_name_is_not_a_hit() {
    let (_tree, names) = PlantedNames::names();

    assert!(DuplicateScan::scan_text(&names, Path::new("a.rs"), "struct TextButtonRow;").is_empty());
    assert!(DuplicateScan::scan_text(&names, Path::new("a.rs"), "pub struct ScreenshotToast;").is_empty());
}

#[test]
fn a_use_line_is_not_a_hit() {
    let (_tree, names) = PlantedNames::names();

    assert!(DuplicateScan::scan_text(&names, Path::new("a.rs"), "use amml_ui::widget::Toast;").is_empty());
    assert!(DuplicateScan::scan_text(&names, Path::new("a.rs"), "// struct Toast was here").is_empty());
}

#[test]
fn a_hit_in_a_nested_file_is_found() {
    let (_names_tree, names) = PlantedNames::names();
    let tree = SourceTree::new();
    tree.file("views/viewer/mod.rs", "mod toast;\n");
    tree.file("views/viewer/toast.rs", "pub struct Toast;\n");

    let hits = DuplicateScan::scan(&names, &SourceFiles::new(tree.root())).unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(hits.len(), 1);
    assert!(hits[0].file.ends_with("views/viewer/toast.rs"));
}
