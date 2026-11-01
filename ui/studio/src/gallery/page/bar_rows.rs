use amml_ui::draw::Draw;
use amml_ui::gallery::SheetGrid;
use amml_ui::tokens::{ColorToken, color};
use egui::{Frame, Rect, Sense, Ui, UiBuilder, vec2};

/// The chrome of the bar pages: like Paper's bar artboards, a column of full-width rows on `--color-bg-app`, each
/// under a small-caps caption, with no page padding around the bars.
pub struct BarRows;

impl BarRows {
    /// Sheet chrome: the caption's inset and the space around it.
    const CAPTION_INSET: f32 = 24.0;
    const CAPTION_TOP: f32 = 20.0;
    const CAPTION_BOTTOM: f32 = 8.0;

    pub fn page(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
        ui.painter().rect_filled(ui.ctx().content_rect(), 0, <color::BgApp as ColorToken>::VALUE);
        Frame::new().fill(<color::BgApp as ColorToken>::VALUE).inner_margin(Draw::margin(0.0, 0.0)).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            body(ui);
            ui.add_space(Self::CAPTION_TOP);
        });
    }

    /// A caption, then `body` in a row `width` wide at the left edge.
    pub fn row(ui: &mut Ui, caption: &str, width: f32, body: impl FnOnce(&mut Ui)) {
        ui.add_space(Self::CAPTION_TOP);
        ui.horizontal(|ui| {
            ui.add_space(Self::CAPTION_INSET);
            SheetGrid::head(ui, caption);
        });
        ui.add_space(Self::CAPTION_BOTTOM);
        let (rect, _) = ui.allocate_exact_size(vec2(width, 0.0), Sense::hover());
        let place = Rect::from_min_size(rect.min, vec2(width, ui.available_height()));
        let used = ui.scope_builder(UiBuilder::new().max_rect(place), |ui| {
            ui.set_width(width);
            body(ui);
            ui.min_rect()
        });
        ui.allocate_exact_size(vec2(width, used.inner.height()), Sense::hover());
    }
}
