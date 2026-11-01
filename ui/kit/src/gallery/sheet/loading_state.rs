use egui::{Frame, Rect, Ui, vec2};

use crate::draw::{Corner, Draw};
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{LoadingKind, LoadingState};

/// Paper: Widgets, `Widget / Loading state` (`BAA-0`).
pub struct LoadingStateSheet;

impl LoadingStateSheet {
    /// Sheet chrome: the two host panels (the second is `--container-inspector`, an amml-studio-ui token).
    const FULL_WIDTH: f32 = 640.0;
    const INSPECTOR_WIDTH: f32 = 430.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Loading state", |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(SheetGrid::COLUMN_GAP, 0.0);
                SheetGrid::column(ui, "SKELETON ROWS", Self::FULL_WIDTH, |ui| {
                    Self::host(ui, Self::FULL_WIDTH, LoadingState::new(LoadingKind::Skeleton { rows: 4 }));
                });
                SheetGrid::column(ui, "INLINE ROW", Self::INSPECTOR_WIDTH, |ui| {
                    let inline = LoadingState::new(LoadingKind::Inline(Sample::text("Loading replays")));
                    Self::host(ui, Self::INSPECTOR_WIDTH, inline);
                });
            });
        });
    }

    // a host panel `width` wide, padding `--spacing-s`, round `state`
    fn host(ui: &mut Ui, width: f32, state: LoadingState) {
        let padding = <spacing::S as SpacingToken>::VALUE;
        let border = <spacing::Border as SpacingToken>::VALUE;
        Frame::new()
            .fill(<color::BgPanel as ColorToken>::VALUE)
            .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
            .corner_radius(Corner::Panel.radius(Rect::NOTHING))
            .inner_margin(Draw::margin(padding, padding))
            .show(ui, |ui| {
                ui.set_width(width - (padding + border) * 2.0);
                state.show(ui);
            });
    }
}
