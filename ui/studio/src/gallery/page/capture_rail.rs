use amml_i18n::text::{Text, UserText};
use amml_ui::draw::{TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::gallery::SheetGrid;
use amml_ui::icon::Icon;
use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, color};
use egui::{Rect, Sense, Ui, UiBuilder, vec2};

use crate::gallery::GalleryState;
use crate::widgets::{RailIcon, RailTile, RailTimer, RailTimerState, RailTone};

/// Paper: Entry Point, `Components · capture rail & corner buttons` (`A89-0`), "RAIL TILE": each tile state, the
/// record, resume and discard tiles, and the timer live and paused. The corner buttons are the amml-ui
/// `CornerButton`.
pub struct CaptureRailPage;

impl CaptureRailPage {
    /// Paper: the sheet is 1600 wide.
    pub const WIDTH: f32 = 1600.0;
    // Paper: cells are 40 apart with the caption 12 under the tile
    const GAP: f32 = 40.0;
    const CAPTION_GAP: f32 = 12.0;
    const CELL: f32 = 96.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Capture rail", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::head(ui, "RAIL TILE");
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(Self::GAP, 0.0);
                    let pause = RailIcon::Glyph(Icon::Pause);
                    let tiles = [
                        ("Default", pause, "Pause", RailTone::Normal, InteractionState::Rest),
                        ("Hover", pause, "Pause", RailTone::Normal, InteractionState::Hover),
                        ("Pressed", pause, "Pause", RailTone::Normal, InteractionState::Pressed),
                        ("Focus", pause, "Pause", RailTone::Normal, InteractionState::Focused),
                        ("Disabled", pause, "Pause", RailTone::Normal, InteractionState::Disabled),
                        ("Record", RailIcon::RecordDot, "Record", RailTone::Normal, InteractionState::Rest),
                        ("Resume", RailIcon::Glyph(Icon::Play), "Resume", RailTone::Active, InteractionState::Rest),
                        (
                            "Discard hover",
                            RailIcon::Glyph(Icon::Delete),
                            "Discard",
                            RailTone::Danger,
                            InteractionState::Hover,
                        ),
                    ];
                    for (caption, icon, label, tone, state) in tiles {
                        Self::cell(ui, caption, |ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(RailTile::SIDE, RailTile::SIDE), Sense::hover());
                            RailTile::paint(ui.painter(), rect, icon, &Self::words(label), tone, state);
                        });
                    }
                    for (caption, state) in
                        [("Timer · live", RailTimerState::Live), ("Timer · paused", RailTimerState::Paused)]
                    {
                        Self::cell(ui, caption, |ui| {
                            RailTimer::show(ui, state, 761_000);
                        });
                    }
                });
            });
        });
    }

    // a sample label, as a caller gets it from its model
    fn words(text: &str) -> Text {
        Text::user(UserText::from_model(text.to_owned()))
    }

    // a tile and its caption under it, in a column `CELL` wide
    fn cell(ui: &mut Ui, caption: &str, body: impl FnOnce(&mut Ui)) {
        ui.vertical(|ui| {
            ui.set_width(Self::CELL);
            ui.spacing_mut().item_spacing = vec2(0.0, Self::CAPTION_GAP);
            body(ui);
            let style = TextStyle::new(TypeSize::Note, FontFace::Regular);
            let dim = <color::TextDim as ColorToken>::VALUE;
            let size = vec2(Self::CELL, style.size.line_height());
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            let mut child = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(rect.min, size)));
            SheetGrid::label(&mut child, caption, style, dim);
        });
    }
}
