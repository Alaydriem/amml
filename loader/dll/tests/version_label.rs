use std::path::Path;

use amml::VersionLabel;

#[test]
fn the_label_names_amml_and_the_instance() {
    let label = VersionLabel::new("1.26.63.01-hotbar");

    assert_eq!(label.text(), "AMML / 1.26.63.01-hotbar");
}

#[test]
fn the_instance_is_the_name_of_the_game_folder() {
    let game = Path::new(r"C:\Users\me\AppData\Local\amml\instances\1.26.63.01-dynlight");

    assert_eq!(VersionLabel::instance_name(game), "1.26.63.01-dynlight");
}

#[test]
fn a_game_folder_without_a_name_is_unknown() {
    assert_eq!(VersionLabel::instance_name(Path::new("")), "unknown");
}
