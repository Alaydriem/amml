use amml_gdk::FileUri;
use std::path::Path;

#[test]
fn encodes_spaces_and_reserved_characters() {
    let uri = FileUri::from_path(Path::new(r"C:\Users\o'brien\amml\launcher\downloads\a b#c%.msixvc"));
    assert_eq!(uri, "file:///C:/Users/o%27brien/amml/launcher/downloads/a%20b%23c%25.msixvc");
}

#[test]
fn drops_the_extended_length_prefix() {
    assert_eq!(FileUri::from_path(Path::new(r"\\?\C:\x\y.msixvc")), "file:///C:/x/y.msixvc");
}
