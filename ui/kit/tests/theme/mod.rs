use amml_ui::font::{FontFace, FontSource};
use amml_ui::text::UiLocale;
use amml_ui::theme::Theme;
use amml_ui::tokens::{ColorToken, color};
use egui::{Context, RawInput};

#[test]
fn install_registers_fonts_english_and_token_visuals() {
    let ctx = Context::default();
    Theme::install(&ctx, FontSource::BuiltIn);
    let mut output = ctx.run_ui(RawInput::default(), |_| {});
    output.textures_delta.clear();
    let has_semibold = ctx.fonts(|fonts| fonts.definitions().families.contains_key(&FontFace::Semibold.family()));
    assert!(has_semibold);
    assert_eq!(UiLocale::get(&ctx).locale(), None);
    assert_eq!(ctx.global_style().visuals.panel_fill, <color::BgPanel as ColorToken>::VALUE);
    assert!(!ctx.options(|options| options.zoom_with_keyboard));
}
