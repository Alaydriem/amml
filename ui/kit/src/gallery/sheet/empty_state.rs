use egui::{Rect, Sense, Ui, UiBuilder, vec2};

use crate::draw::{Corner, Draw};
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::icon::Icon;
use crate::tokens::{ColorToken, color};
use crate::widget::{ButtonKind, EmptyState, Leading, TextButton};

/// Paper: Widgets, `Widget / Empty state` (`B7U-0`).
pub struct EmptyStateSheet;

impl EmptyStateSheet {
    /// Sheet chrome: the two host panels (the second is `--container-inspector`, an amml-studio-ui token).
    const FULL_WIDTH: f32 = 640.0;
    const INSPECTOR_WIDTH: f32 = 430.0;
    const HOST_HEIGHT: f32 = 320.0;
    const HOST_PADDING: f32 = 24.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Empty state", |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(SheetGrid::COLUMN_GAP, 0.0);
                SheetGrid::column(ui, "FULL PANEL", Self::FULL_WIDTH, |ui| {
                    Self::host(ui, Self::FULL_WIDTH, |ui| {
                        EmptyState::new(
                            Icon::Folder,
                            Sample::text("No replays yet"),
                            Sample::text("Recorded sessions show up here."),
                        )
                        .action(
                            TextButton::new(ButtonKind::Secondary, Sample::text("Open replays folder"))
                                .leading(Leading::Icon(Icon::Folder)),
                        )
                        .show(ui);
                    });
                });
                SheetGrid::column(ui, "INSPECTOR COLUMN", Self::INSPECTOR_WIDTH, |ui| {
                    Self::host(ui, Self::INSPECTOR_WIDTH, |ui| {
                        EmptyState::new(
                            Icon::Camera,
                            Sample::text("No keys on the timeline"),
                            Sample::text("Add a camera key to start a scene."),
                        )
                        .action(
                            TextButton::new(ButtonKind::Secondary, Sample::text("Add camera key"))
                                .leading(Leading::Icon(Icon::Add)),
                        )
                        .show(ui);
                    });
                });
            });
        });
    }

    // a host panel `width` x 320, padding 24; `content` gets the rect inside the padding
    fn host(ui: &mut Ui, width: f32, content: impl FnOnce(&mut Ui)) {
        let (rect, _) = ui.allocate_exact_size(vec2(width, Self::HOST_HEIGHT), Sense::hover());
        Draw::fill(ui.painter(), rect, Corner::Panel, <color::BgPanel as ColorToken>::VALUE);
        Draw::stroke(ui.painter(), rect, Corner::Panel, <color::Line as ColorToken>::VALUE);
        let inside: Rect = rect.shrink(Self::HOST_PADDING);
        ui.scope_builder(UiBuilder::new().max_rect(inside), content);
    }
}
