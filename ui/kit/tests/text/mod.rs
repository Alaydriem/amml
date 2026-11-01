mod amml_ui_packs;

use amml_i18n::text::{Text, UserText};
use amml_ui::AmmlUiMessage;
use amml_ui::text::{UiLocale, UiText};
use egui::Context;

use crate::support::PackFixture;

#[test]
fn english_when_unset() {
    let ctx = Context::default();
    assert_eq!(UiLocale::get(&ctx).locale(), None);
    assert_eq!(UiText::resolve(&ctx, &Text::message(AmmlUiMessage::Close)), "Close");
}

#[test]
fn resolve_reads_the_installed_pack() {
    let ctx = Context::default();
    UiLocale::set(&ctx, PackFixture::localizer(&[("Close", "Çľöšé")]));
    assert_eq!(UiText::resolve(&ctx, &Text::message(AmmlUiMessage::Close)), "Çľöšé");
}

#[test]
fn user_text_is_shown_as_it_is() {
    let ctx = Context::default();
    UiLocale::set(&ctx, PackFixture::localizer(&[("Close", "Çľöšé")]));
    let text = Text::user(UserText::from_model("Close".to_owned()));
    assert_eq!(UiText::resolve(&ctx, &text), "Close");
}
