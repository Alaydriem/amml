use amml_ui::gallery::SheetGrid;
use amml_ui::tokens::{ColorToken, ContainerToken, color};
use egui::{Id, Rect, Sense, Ui, UiBuilder, pos2, vec2};

use super::{BarRows, TimelineSamples, ViewerSamples};
use crate::component::inspector::{InspectorLocked, InspectorTab, InspectorTabs, UndoHistoryPopover, UndoHistoryView};
use crate::gallery::GalleryState;
use crate::id::StudioUiId;
use crate::tokens::container;

/// Paper: Menus + Dialogs, `Key active` (`4JQ-0`), `Locked` (`4MU-0`) and `Undo history` (`46V-0`). The inspector's
/// tab strip in each state, a locked body, and the undo history empty and after six edits. The tab contents are the
/// host's (Plan 08), so the locked body is drawn empty.
pub struct InspectorPage;

impl InspectorPage {
    /// Paper: the inspector sheets are 430 wide; the undo sheet 720.
    pub const WIDTH: f32 = 720.0;
    // sheet chrome: the locked body's height, and the column gap of the undo sheet
    const BODY: f32 = 260.0;
    const GAP: f32 = 48.0;

    pub fn tabs(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            let states = [
                ("KEY ACTIVE", InspectorTab::Key),
                ("SCENE ACTIVE", InspectorTab::Scene),
                ("PLAYERS ACTIVE", InspectorTab::Players),
            ];
            for (index, (caption, tab)) in states.into_iter().enumerate() {
                ViewerSamples::row(ui, caption, |ui| {
                    Self::column(ui, |ui| {
                        InspectorTabs::new(tab).id(Id::unique(StudioUiId::InspectorTabs).with(index)).show(ui);
                    });
                });
            }
        });
    }

    pub fn locked(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            ViewerSamples::row(ui, "LOCKED · SCENE CAMERA DRIVES", |ui| {
                Self::column(ui, |ui| {
                    InspectorTabs::new(InspectorTab::Key).show(ui);
                    let (body, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::BODY), Sense::hover());
                    let reason = TimelineSamples::words("Locked: Scene camera drives · pause to edit");
                    InspectorLocked::new(reason).paint(ui.painter(), body);
                });
            });
        });
    }

    pub fn undo_history(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Undo history", |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(Self::GAP, 0.0);
                let empty = UndoHistoryView { steps: Vec::new() };
                let names = ["Easing", "Move keys", "Edit key", "Add key", "Scene settings", "Rename scene"];
                let six = UndoHistoryView { steps: names.iter().map(|name| TimelineSamples::words(name)).collect() };
                for (head, view) in [("EMPTY STACK", &empty), ("AFTER 6 EDITS", &six)] {
                    SheetGrid::column(ui, head, UndoHistoryPopover::WIDTH, |ui| {
                        UndoHistoryPopover::new(view).show(ui);
                    });
                }
            });
        });
    }

    // a `--container-inspector` column on `--color-bg-panel` with a `--color-line` left edge
    fn column(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
        let width = <container::Inspector as ContainerToken>::VALUE;
        let rect = Rect::from_min_size(ui.cursor().min, vec2(width, ui.available_height()));
        // the scope moves the page's cursor past what `body` allocates
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            let painter = ui.painter().clone();
            let index = painter.add(egui::Shape::Noop);
            body(ui);
            let used = Rect::from_min_max(rect.min, pos2(rect.right(), ui.min_rect().bottom()));
            painter.set(index, egui::Shape::rect_filled(used, 0, <color::BgPanel as ColorToken>::VALUE));
            let line = egui::Stroke::new(1.0, <color::Line as ColorToken>::VALUE);
            painter.vline(used.left() + 0.5, used.y_range(), line);
        });
    }
}
