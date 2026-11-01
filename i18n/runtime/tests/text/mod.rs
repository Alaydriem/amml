mod placeholder_format;

use std::borrow::Cow;

use amml_i18n::localize::Localizer;
use amml_i18n::message::Message;
use amml_i18n::pack::LanguagePack;
use amml_i18n::text::{Args, Text, UserText};

use crate::fixture::SampleMessage;

fn russian() -> Localizer {
    let pack: LanguagePack = serde_json::from_str(
        r#"{"v":1,"locale":"ru","plural":["one","few","many"],
            "m":{"Sign In Again":"Войти снова",
                 "Hello, {name}":"Привет, {name}",
                 "{count} player nearby":["{count} игрок","{count} игрока","{count} игроков"]}}"#,
    )
    .unwrap_or_else(|error| panic!("fixture pack is invalid: {error}"));
    Localizer::new("ru", vec![(SampleMessage::DOMAIN, pack)])
}

#[test]
fn plain_message_borrows() {
    let ru = russian();
    let text = Text::message(SampleMessage::SignInAgain);
    let resolved = text.resolve(&ru);
    assert!(matches!(resolved, Cow::Borrowed("Войти снова")));
}

#[test]
fn format_replaces_named_placeholders() {
    let ru = russian();
    let text = Text::format(SampleMessage::Greeting, Args::new().user("name", UserText::from_model("Alex".into())));
    assert_eq!(text.resolve(&ru), "Привет, Alex");
    assert_eq!(text.resolve(&Localizer::english()), "Hello, Alex");
}

#[test]
fn missing_arg_is_left_visible() {
    let text = Text::format(SampleMessage::Greeting, Args::new().int("count", 2));
    assert_eq!(text.resolve(&Localizer::english()), "Hello, {name}");
}

#[test]
fn plural_uses_count_and_args() {
    let ru = russian();
    let text = Text::plural(SampleMessage::PlayersNearby, 3, Args::new());
    assert_eq!(text.resolve(&ru), "3 игрока");
    assert_eq!(text.resolve(&Localizer::english()), "3 players nearby");
    assert_eq!(
        Text::plural(SampleMessage::PlayersNearby, 1, Args::new()).resolve(&Localizer::english()),
        "1 player nearby"
    );
}

#[test]
fn an_explicit_count_arg_wins_over_the_plural_count() {
    let text = Text::plural(
        SampleMessage::PlayersNearby,
        1000,
        Args::new().user("count", UserText::from_model("1 000".into())),
    );
    assert_eq!(text.resolve(&Localizer::english()), "1 000 players nearby");
}

#[test]
fn a_message_arg_is_translated_too() {
    let ru = russian();
    let text = Text::format(SampleMessage::Greeting, Args::new().message("name", SampleMessage::SignInAgain));
    assert_eq!(text.resolve(&ru), "Привет, Войти снова");
}

#[test]
fn user_text_passes_through_unchanged() {
    let ru = russian();
    let text = Text::user(UserText::from_model("Sign In Again".into()));
    assert!(matches!(text.resolve(&ru), Cow::Borrowed("Sign In Again")));
}
