use std::collections::HashMap;

use amml_i18n::pack::{LanguagePack, PackEntry};
use amml_i18n_tool::report::{Coverage, CoverageReport};

const POT: &str = r#"msgid ""
msgstr ""
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

msgid "Save"
msgstr ""

msgctxt "audio"
msgid "Output"
msgstr ""
"#;

fn coverage(translated: usize, total: usize) -> Coverage {
    Coverage::new(translated, total)
}

#[test]
fn coverage_counts_pot_messages_found_in_the_pack() {
    let mut m = HashMap::new();
    m.insert("audio\u{4}Output".to_string(), PackEntry::One("Sortie".into()));
    m.insert("Gone From The Pot".to_string(), PackEntry::One("x".into()));
    let pack = LanguagePack { v: 1, locale: "fr".into(), plural: vec!["one".into(), "other".into()], m };
    let result = CoverageReport::for_pack(POT, &pack).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!((result.translated, result.total), (1, 2));
    assert_eq!(result.percent, 50.0);
}

#[test]
fn ninety_percent_ships_eighty_nine_does_not() {
    assert!(coverage(90, 100).ships());
    assert!(!coverage(89, 100).ships());
    assert!(!coverage(899, 1000).ships());
}

#[test]
fn an_empty_catalog_is_complete_rather_than_dividing_by_zero() {
    assert!(coverage(0, 0).ships());
}

#[test]
fn a_locale_at_or_above_the_gate_is_marked_as_shipping() {
    let body = CoverageReport::render(&[("de".into(), coverage(95, 100)), ("ru".into(), coverage(50, 100))]);
    assert!(body.contains("| `de` | 95 / 100 | 95% | yes |"), "{body}");
    assert!(body.contains("| `ru` | 50 / 100 | 50% | no |"), "{body}");
}

#[test]
fn an_empty_list_explains_the_fallback_rather_than_showing_an_empty_table() {
    assert!(CoverageReport::render(&[]).contains("No `.po` files yet"));
}
