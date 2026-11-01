use amml_i18n::text::Text;
use egui::Context;

use super::UiLocale;

/// Resolves `Text` in the locale of an egui context.
pub struct UiText;

impl UiText {
    pub fn resolve(ctx: &Context, text: &Text) -> String {
        let localizer = UiLocale::get(ctx);
        text.resolve(&localizer).into_owned()
    }
}
