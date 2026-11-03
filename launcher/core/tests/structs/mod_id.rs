use amml_launcher_core::{LauncherError, ModId};
use std::path::Path;

#[test]
fn ids_with_path_parts_or_capitals_are_refused() {
    for bad in ["@../x", "@a/b/c", "@A/b", "a/b", "@a/", "@a/..", "@/b", "@a/b.c", "@-a/b", "@a/b c", ""] {
        assert!(matches!(ModId::parse(bad), Err(LauncherError::BadModId(_))), "{bad}");
    }
    let long = format!("@a/{}", "x".repeat(65));
    assert!(matches!(ModId::parse(&long), Err(LauncherError::BadModId(_))));
    assert!(ModId::parse(&format!("@a/{}", "x".repeat(64))).is_ok());
}

#[test]
fn a_valid_id_maps_to_its_document_and_folder() {
    let id = ModId::parse("@alaydriem/zoom").unwrap();
    assert_eq!(id.document_path(), "mods/@alaydriem/zoom.json");
    assert_eq!(id.folder(Path::new("mods")), Path::new("mods").join("@alaydriem").join("zoom"));
    assert_eq!(id.as_str(), "@alaydriem/zoom");
}
