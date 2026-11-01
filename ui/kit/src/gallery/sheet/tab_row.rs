use egui::{Id, Rect, Sense, Ui, vec2};

use crate::draw::{Corner, Draw};
use crate::gallery::{GalleryId, GalleryState, Sample, SheetGrid};
use crate::state::InteractionState;
use crate::tokens::{ColorToken, ContainerToken, SpacingToken, color, container, spacing};
use crate::widget::{TabRow, TabRowSize};

/// Paper: Widgets, `Widget / Tab row` (`AUS-0`).
pub struct TabRowSheet;

impl TabRowSheet {
    const COLUMN: f32 = 140.0;
    /// Sheet chrome: the inspector example is `--container-inspector` wide, an amml-studio-ui token, so
    /// the sheet keeps the number.
    const INSPECTOR_WIDTH: f32 = 430.0;
    // (state, selected) per column: REST, HOVER, PRESSED, SELECTED, FOCUS, DISABLED
    const COLUMNS: [(InteractionState, bool); 6] = [
        (InteractionState::Rest, false),
        (InteractionState::Hover, false),
        (InteractionState::Pressed, false),
        (InteractionState::Rest, true),
        (InteractionState::Focused, false),
        (InteractionState::Disabled, false),
    ];

    pub fn show(ui: &mut Ui, state: &mut GalleryState) {
        SheetGrid::page(ui, "Tab row", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::COLUMN, &["REST", "HOVER", "PRESSED", "SELECTED", "FOCUS", "DISABLED"]);
                for (label, size, words) in
                    [("Inspector", TabRowSize::Inspector, "Scene"), ("Compact", TabRowSize::Compact, "Replays")]
                {
                    SheetGrid::row(ui, label, Self::COLUMN, size.height(), Self::COLUMNS.len(), |ui, index, cell| {
                        let text = Sample::text(words);
                        let width = TabRow::tab_width(ui.ctx(), &text, size);
                        let (tab_state, selected) = Self::COLUMNS[index];
                        let rect = SheetGrid::place(cell, vec2(width, size.height()));
                        TabRow::paint_tab(ui.painter(), rect, &text, size, tab_state, selected);
                    });
                }
            });
            SheetGrid::section(ui, "IN A HEADER", |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(SheetGrid::COLUMN_GAP, 0.0);
                    Self::header(ui, Self::INSPECTOR_WIDTH, |ui| {
                        let tabs = vec![Sample::text("Key"), Sample::text("Scene"), Sample::text("Players")];
                        TabRow::new(Id::unique(GalleryId::InspectorTabs), &mut state.inspector_tab, tabs).show(ui);
                    });
                    let dialog = <container::Dialog as ContainerToken>::VALUE;
                    Self::header(ui, dialog, |ui| {
                        let tabs = vec![Sample::text("Replays"), Sample::text("Settings")];
                        TabRow::new(Id::unique(GalleryId::LibraryTabs), &mut state.library_tab, tabs)
                            .size(TabRowSize::Compact)
                            .show(ui);
                    });
                });
            });
        });
    }

    // a panel `width` wide: `--spacing-s` above the strip, then a `--spacing-lane` empty body
    fn header(ui: &mut Ui, width: f32, strip: impl FnOnce(&mut Ui)) {
        let top = <spacing::S as SpacingToken>::VALUE;
        egui::Frame::new()
            .fill(<color::BgPanel as ColorToken>::VALUE)
            .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
            .corner_radius(Corner::Panel.radius(Rect::NOTHING))
            .inner_margin(egui::Margin { left: 0, right: 0, top: Draw::margin(0.0, top).top, bottom: 0 })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(width);
                    ui.set_max_width(width);
                    ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                    strip(ui);
                    ui.allocate_exact_size(vec2(width, <spacing::Lane as SpacingToken>::VALUE), Sense::hover());
                });
            });
    }
}
