use egui::{Frame, Rect, Ui, vec2};

use crate::draw::{Corner, Draw};
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::tokens::{ColorToken, color};
use crate::widget::Caption;

/// Paper: Widgets, `Widget / Caption`. A one line caption and a wrapped one, each under its head,
/// in a 260 wide host panel.
pub struct CaptionSheet;

impl CaptionSheet {
    /// Sheet chrome: the host panel's width and padding, and the gap between its parts.
    const HOST_WIDTH: f32 = 260.0;
    const HOST_PADDING: f32 = 16.0;
    const PART_GAP: f32 = 10.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Caption", |ui| {
            Frame::new()
                .fill(<color::BgPanel as ColorToken>::VALUE)
                .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
                .corner_radius(Corner::Panel.radius(Rect::NOTHING))
                .inner_margin(Draw::margin(Self::HOST_PADDING, Self::HOST_PADDING))
                .show(ui, |ui| {
                    ui.set_width(Self::HOST_WIDTH - 2.0 * Self::HOST_PADDING);
                    ui.spacing_mut().item_spacing = vec2(0.0, Self::PART_GAP);
                    SheetGrid::head(ui, "ONE LINE");
                    Caption::new(Sample::text("Saved automatically")).show(ui);
                    SheetGrid::head(ui, "WRAPPED");
                    Caption::new(Sample::text("Or drop files into mods/alaydriem-stratum/data/import")).show(ui);
                });
        });
    }
}
