use amml_ui::font::{FontFace, FontLoader, FontSource};
use amml_ui::tokens::{FontFamilyToken, FontWeightToken, font_family, font_weight};
use egui::{Align2, Color32, FontFamily, pos2};

use crate::support::PaintCapture;

const FACES: [FontFace; 3] = [FontFace::Regular, FontFace::Semibold, FontFace::Icons];

#[test]
fn built_in_source_registers_every_face() {
    let definitions = FontLoader::definitions(FontSource::BuiltIn);
    for face in FACES {
        let chain = definitions.families.get(&face.family());
        assert!(chain.is_some_and(|chain| !chain.is_empty()), "{face:?} has no fonts");
    }
}

#[test]
fn embedded_source_puts_nordwand_first() {
    let definitions = FontLoader::definitions(FontSource::Embedded);
    let first = |face: FontFace| definitions.families.get(&face.family()).and_then(|chain| chain.first().cloned());
    assert_eq!(first(FontFace::Regular).as_deref(), Some(FontLoader::TEXT_KEY));
    assert_eq!(first(FontFace::Semibold).as_deref(), Some(FontLoader::SEMIBOLD_KEY));
}

#[test]
fn face_names_follow_the_font_tokens() {
    let icons = <font_family::Icons as FontFamilyToken>::VALUE;
    assert_eq!(FontFace::Icons.family(), FontFamily::Name(icons.into()));
    let semibold = format!(
        "{} {}",
        <font_family::Ui as FontFamilyToken>::VALUE,
        <font_weight::Semibold as FontWeightToken>::VALUE
    );
    assert_eq!(FontFace::Semibold.family(), FontFamily::Name(semibold.into()));
    assert_eq!(FontFace::Regular.family(), FontFamily::Proportional);
}

#[test]
fn register_keeps_families_already_bound() {
    let mut definitions = egui::FontDefinitions::default();
    let other = FontFamily::Name("ui-icons".into());
    definitions.families.insert(other.clone(), vec!["other-font".to_owned()]);
    FontLoader::register(&mut definitions, FontSource::BuiltIn, std::path::Path::new("."));
    assert_eq!(definitions.families.get(&other), Some(&vec!["other-font".to_owned()]));
    for face in FACES {
        assert!(definitions.families.contains_key(&face.family()), "{face:?} was not registered");
    }
}

#[test]
fn icon_font_falls_back_to_segoe_mdl2() {
    let Ok(folder) = tempfile::tempdir() else { panic!("no temporary folder") };
    let registered = |definitions: &egui::FontDefinitions| definitions.font_data.contains_key(FontLoader::ICONS_KEY);
    let mut empty = egui::FontDefinitions::default();
    FontLoader::register(&mut empty, FontSource::Embedded, folder.path());
    assert!(!registered(&empty), "an empty folder has no icon font");
    if let Err(error) = std::fs::write(folder.path().join("segmdl2.ttf"), b"font") {
        panic!("could not write the stand-in font: {error}");
    }
    let mut with_mdl2 = egui::FontDefinitions::default();
    FontLoader::register(&mut with_mdl2, FontSource::Embedded, folder.path());
    assert!(registered(&with_mdl2), "Segoe MDL2 Assets was not used");
}

#[test]
fn font_folder_follows_windir() {
    let folder = FontLoader::font_dir();
    assert!(folder.ends_with("Fonts"), "{folder:?}");
    if let Some(windows) = std::env::var_os("WINDIR") {
        assert!(folder.starts_with(windows), "{folder:?} is not under WINDIR");
    }
}

#[test]
fn paint_capture_sees_text() {
    let shapes = PaintCapture::shapes(|painter| {
        painter.text(pos2(50.0, 50.0), Align2::CENTER_CENTER, "x", FontFace::Semibold.id(12.0), Color32::WHITE);
    });
    assert_eq!(PaintCapture::texts(&shapes), vec![("x".to_owned(), 12.0, FontFace::Semibold.family())]);
}
