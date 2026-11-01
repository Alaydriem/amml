use amml_i18n::pack::{LanguagePack, PackEntry};
use amml_i18n_tool::pseudo::PseudoLocale;

const POT: &str = r#"msgid ""
msgstr ""
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

msgid "Sign In"
msgstr ""

msgid "{count} player"
msgid_plural "{count} players"
msgstr[0] ""
msgstr[1] ""
"#;

fn pack() -> LanguagePack {
    PseudoLocale::from_pot(POT).unwrap_or_else(|error| panic!("{error}"))
}

fn one(pack: &LanguagePack, key: &str) -> String {
    match pack.m.get(key) {
        Some(PackEntry::One(text)) => text.clone(),
        other => panic!("{key} is {other:?}"),
    }
}

fn many(pack: &LanguagePack, key: &str) -> Vec<String> {
    match pack.m.get(key) {
        Some(PackEntry::Many(forms)) => forms.clone(),
        other => panic!("{key} is {other:?}"),
    }
}

#[test]
fn every_message_is_bracketed_so_an_unextracted_string_is_visible() {
    let text = one(&pack(), "Sign In");
    assert!(text.starts_with('[') && text.ends_with(']'), "{text}");
}

#[test]
fn the_text_is_expanded_so_tight_layouts_fail_here_rather_than_in_german() {
    let text = one(&pack(), "Sign In");
    assert!(text.chars().count() as f64 > "Sign In".len() as f64 * 1.3, "{text}");
}

#[test]
fn pseudo_keeps_placeholders() {
    let forms = many(&pack(), "{count} player");
    assert_eq!(forms.len(), 2);
    assert!(forms.iter().all(|form| form.contains("{count}")));
}

#[test]
fn the_accenting_reaches_the_words_around_a_placeholder() {
    let forms = many(&pack(), "{count} player");
    assert!(!forms[0].contains("player"));
}

#[test]
fn plural_messages_get_one_pseudo_form_per_category() {
    let pack = pack();
    assert_eq!(many(&pack, "{count} player").len(), pack.plural.len());
}

#[test]
fn the_locale_is_en_xa_so_it_never_collides_with_a_real_one() {
    assert_eq!(pack().locale, "en_XA");
}
