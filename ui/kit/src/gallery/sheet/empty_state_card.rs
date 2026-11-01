use egui::{Frame, Rect, Ui};

use crate::draw::{Corner, Draw};
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::EmptyState;

/// Paper: Widgets, `Widget / Empty state` (`B7U-0`), the state `Card, no icon`.
pub struct EmptyStateCardSheet;

impl EmptyStateCardSheet {
    /// Sheet chrome: the host column is `--container-inspector` wide, an amml-studio-ui token.
    const COLUMN_WIDTH: f32 = 430.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let padding = <spacing::L as SpacingToken>::VALUE;
        let border = <spacing::Border as SpacingToken>::VALUE;
        SheetGrid::page(ui, "Empty state", |ui| {
            SheetGrid::column(ui, "CARD, NO ICON", Self::COLUMN_WIDTH, |ui| {
                Frame::new()
                    .fill(<color::BgPanel as ColorToken>::VALUE)
                    .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
                    .corner_radius(Corner::Panel.radius(Rect::NOTHING))
                    .inner_margin(Draw::margin(padding, padding))
                    .show(ui, |ui| {
                        ui.set_width(Self::COLUMN_WIDTH - 2.0 * (padding + border));
                        let words = "Click a key on the timeline. Shift-click or drag a box for more. Right-click a lane: \u{201C}Add key here\u{201D}. Click a track name for the track inspector.";
                        EmptyState::card(Sample::text("No key selected"), Sample::text(words)).show(ui);
                    });
            });
        });
    }
}
