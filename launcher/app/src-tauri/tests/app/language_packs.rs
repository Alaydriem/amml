use amml_launcher_app_lib::app::LanguagePacks;

fn pack(locale: &str, greeting: &str) -> String {
    format!(r#"{{"v":1,"locale":"{locale}","plural":["one","other"],"m":{{"Hello":"{greeting}"}}}}"#)
}

#[test]
fn the_locales_with_a_pack_are_listed_sorted() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("fr.json"), pack("fr", "Bonjour")).unwrap();
    std::fs::write(dir.path().join("de.json"), pack("de", "Hallo")).unwrap();
    std::fs::write(dir.path().join("notes.txt"), "not a pack").unwrap();
    let packs = LanguagePacks::new(dir.path().to_path_buf());

    assert_eq!(packs.available(), vec!["de".to_string(), "fr".to_string()]);
}

#[test]
fn a_regional_request_gets_the_pack_of_its_language() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("de.json"), pack("de", "Hallo")).unwrap();
    let packs = LanguagePacks::new(dir.path().to_path_buf());

    let loaded = packs.load_for("de-AT").unwrap();

    assert_eq!(loaded.locale, "de");
    assert_eq!(loaded.lookup("Hello"), Some("Hallo"));
}

#[test]
fn english_and_unknown_locales_get_no_pack() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("de.json"), pack("de", "Hallo")).unwrap();
    let packs = LanguagePacks::new(dir.path().to_path_buf());

    assert!(packs.load_for("en-US").is_none());
    assert!(packs.load_for("ja").is_none());
}

#[test]
fn a_malformed_pack_falls_back_to_english() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("de.json"), "{ not json").unwrap();
    let packs = LanguagePacks::new(dir.path().to_path_buf());

    assert!(packs.load_for("de").is_none());
}

#[test]
fn a_missing_folder_has_no_locales() {
    let dir = tempfile::tempdir().unwrap();
    let packs = LanguagePacks::new(dir.path().join("none"));

    assert!(packs.available().is_empty());
    assert!(packs.load_for("de").is_none());
}
