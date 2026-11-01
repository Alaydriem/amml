use amml_ui_rules::source::SourceFiles;
use amml_ui_rules::widget_name::WidgetNames;

use crate::support::SourceTree;

#[test]
fn reads_public_structs_and_enums_in_nested_files() {
    let tree = SourceTree::new();
    tree.file("button/mod.rs", "pub enum ButtonKind {\n    Primary,\n}\n");
    tree.file("button/text_button.rs", "/// A button.\npub struct TextButton<'a> {\n    text: &'a str,\n}\n");
    tree.file("chip/key_cap.rs", "pub struct KeyCap;\nstruct Hidden;\npub(crate) struct Inner;\n");

    let names = WidgetNames::from_source(&SourceFiles::new(tree.root())).unwrap_or_else(|error| panic!("{error}"));

    assert!(names.contains("ButtonKind"));
    assert!(names.contains("TextButton"));
    assert!(names.contains("KeyCap"));
    assert!(!names.contains("Hidden"));
    assert!(!names.contains("Inner"));
    assert_eq!(names.len(), 3);
}

#[test]
fn amml_ui_names_include_text_button_and_toast() {
    let names = WidgetNames::amml_ui().unwrap_or_else(|error| panic!("{error}"));

    assert!(names.contains("TextButton"));
    assert!(names.contains("Toast"));
    assert!(names.contains("Surface"));
    assert!(names.len() > 50, "only {} names: the amml-ui widget folder was not found", names.len());
}
