use amml_i18n::localize::Localizer;
use amml_i18n::message::{Domain, Message};
use amml_i18n::pack::LanguagePack;

use crate::fixture::SampleMessage;

// compiles only for a type that can be shared between threads
struct SharedBetweenThreads<T: Send + Sync>(std::marker::PhantomData<T>);

#[test]
fn localizer_can_be_shared_between_threads() {
    // egui keeps the localizer in its context data, which needs Send + Sync
    let _ = SharedBetweenThreads::<Localizer>(std::marker::PhantomData);
    let shared = std::sync::Arc::new(localizer(russian()));
    let other = std::sync::Arc::clone(&shared);
    let text = std::thread::spawn(move || other.table::<SampleMessage>().text(SampleMessage::SignInAgain).to_owned())
        .join()
        .unwrap_or_else(|_| panic!("the thread panicked"));
    assert_eq!(text, "Войти снова");
}

const OTHER_MOD: Domain = Domain::new("other-mod");

fn pack(json: &str) -> LanguagePack {
    serde_json::from_str(json).unwrap_or_else(|error| panic!("fixture pack is invalid: {error}"))
}

fn russian() -> LanguagePack {
    pack(
        r#"{"v":1,"locale":"ru","plural":["one","few","many"],
            "m":{"Sign In Again":"Войти снова",
                 "audio\u0004Output":"Выход",
                 "{count} player nearby":["игрок","игрока","игроков"]}}"#,
    )
}

fn localizer(pack: LanguagePack) -> Localizer {
    let locale = pack.locale.clone();
    Localizer::new(&locale, vec![(SampleMessage::DOMAIN, pack)])
}

#[test]
fn with_no_pack_a_message_id_is_its_own_english_translation() {
    let english = Localizer::english();
    assert_eq!(english.table::<SampleMessage>().text(SampleMessage::SignInAgain), "Sign In Again");
}

#[test]
fn a_loaded_pack_translates_a_known_message() {
    let ru = localizer(russian());
    assert_eq!(ru.table::<SampleMessage>().text(SampleMessage::SignInAgain), "Войти снова");
}

#[test]
fn missing_entry_falls_back_to_english() {
    let ru = localizer(russian());
    assert_eq!(ru.table::<SampleMessage>().text(SampleMessage::NeverTranslated), "Never Translated");
}

#[test]
fn context_entry_is_found_with_separator_key() {
    let ru = localizer(russian());
    assert_eq!(ru.table::<SampleMessage>().text(SampleMessage::Output), "Выход");
}

#[test]
fn a_pack_for_another_domain_does_not_translate_this_one() {
    let ru = Localizer::new("ru", vec![(OTHER_MOD, russian())]);
    assert_eq!(ru.table::<SampleMessage>().text(SampleMessage::SignInAgain), "Sign In Again");
}

#[test]
fn domains_declared_by_different_mods_load_side_by_side() {
    let ru = Localizer::new("ru", vec![(OTHER_MOD, russian()), (SampleMessage::DOMAIN, russian())]);
    assert!(ru.has_pack(OTHER_MOD));
    assert!(ru.has_pack(SampleMessage::DOMAIN));
    assert!(!ru.has_pack(Domain::new("not-loaded")));
    assert_eq!(ru.table::<SampleMessage>().text(SampleMessage::SignInAgain), "Войти снова");
}

#[test]
fn plural_ru_uses_three_forms() {
    let ru = localizer(russian());
    let table = ru.table::<SampleMessage>();
    for (count, expected) in [(1, "игрок"), (2, "игрока"), (5, "игроков"), (21, "игрок")] {
        assert_eq!(table.plural(SampleMessage::PlayersNearby, count), expected, "count {count}");
    }
}

#[test]
fn with_no_pack_plurals_use_the_two_english_forms() {
    let english = Localizer::english();
    let table = english.table::<SampleMessage>();
    assert_eq!(table.plural(SampleMessage::PlayersNearby, 1), "{count} player nearby");
    assert_eq!(table.plural(SampleMessage::PlayersNearby, 4), "{count} players nearby");
}

#[test]
fn an_untranslated_plural_uses_the_english_forms() {
    let ru = localizer(pack(r#"{"v":1,"locale":"ru","plural":["one","few","many"],"m":{}}"#));
    let table = ru.table::<SampleMessage>();
    assert_eq!(table.plural(SampleMessage::PlayersNearby, 1), "{count} player nearby");
    assert_eq!(table.plural(SampleMessage::PlayersNearby, 3), "{count} players nearby");
}

// bvc took form 0 for every count when the locale had no plural rules. A locale string
// that ICU cannot parse now uses the English rule: one for 1, other for everything else.
#[test]
fn no_rules_plural_falls_back_to_one_other() {
    let broken = localizer(pack(
        r#"{"v":1,"locale":"!! not a locale !!","plural":["one","other"],
            "m":{"Sign In Again":"Encore","{count} player nearby":["{count} joueur","{count} joueurs"]}}"#,
    ));
    let table = broken.table::<SampleMessage>();
    assert_eq!(table.text(SampleMessage::SignInAgain), "Encore");
    assert_eq!(table.plural(SampleMessage::PlayersNearby, 1), "{count} joueur");
    assert_eq!(table.plural(SampleMessage::PlayersNearby, 7), "{count} joueurs");
}

// Review finding 1: a valid tag with no CLDR data gets ICU's root rules, which call every
// count `other`. That must use the English rule too, not form 0 for every count.
#[test]
fn a_locale_without_cldr_data_falls_back_to_one_other() {
    let unknown = localizer(pack(
        r#"{"v":1,"locale":"xx","plural":["one","other"],
            "m":{"{count} player nearby":["{count} one-form","{count} other-form"]}}"#,
    ));
    let table = unknown.table::<SampleMessage>();
    assert_eq!(table.plural(SampleMessage::PlayersNearby, 1), "{count} one-form");
    assert_eq!(table.plural(SampleMessage::PlayersNearby, 5), "{count} other-form");
}

// Japanese really has one category; the fallback must not change its single form.
#[test]
fn a_one_form_locale_keeps_its_single_form() {
    let ja =
        localizer(pack(r#"{"v":1,"locale":"ja","plural":["other"],"m":{"{count} player nearby":["{count} 人"]}}"#));
    let table = ja.table::<SampleMessage>();
    assert_eq!(table.plural(SampleMessage::PlayersNearby, 1), "{count} 人");
    assert_eq!(table.plural(SampleMessage::PlayersNearby, 4), "{count} 人");
}

#[test]
fn lookup_does_not_allocate() {
    let ru = localizer(russian());
    let table = ru.table::<SampleMessage>();
    let first = table.text(SampleMessage::SignInAgain);
    for _ in 0..1000 {
        assert!(std::ptr::eq(first, ru.table::<SampleMessage>().text(SampleMessage::SignInAgain)));
    }
}
