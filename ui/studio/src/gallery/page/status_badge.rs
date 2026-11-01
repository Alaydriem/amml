use amml_i18n::text::{Text, UserText};
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::gallery::SheetGrid;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Align2, Rect, Sense, Ui, vec2};

use crate::component::coverage::{BadgePlacement, CoverageBadge, PlacedBadge};
use crate::gallery::GalleryState;
use crate::tokens::selector::StatusColor;

/// Paper: Widgets, `Widget / Status badge + Placed badge` (`42V-0`). The five status badges, then a badge placed
/// right of, above and below a control.
pub struct StatusBadgePage;

impl StatusBadgePage {
    /// Sheet chrome: a placement column's width and height.
    const COLUMN: f32 = 280.0;
    const CELL_HEIGHT: f32 = 60.0;
    const ROWS: [(&'static str, StatusColor, &'static [&'static str]); 5] = [
        ("Works", StatusColor::Works, &["C1 WORKS (ImGui)", "A1 WORKS (pack)", "A9 AUTO"]),
        ("Engine-only", StatusColor::Engine, &["C5 ENGINE-ONLY"]),
        ("Planned", StatusColor::Planned, &["R1 PLANNED"]),
        ("Mockup-only", StatusColor::Mockup, &["C9 MOCKUP-ONLY"]),
        ("Missing", StatusColor::Missing, &["D5 MISSING"]),
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Status badge + Placed badge", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, SheetGrid::LABEL_WIDTH, &["BADGE"]);
                for (label, status, badges) in Self::ROWS {
                    SheetGrid::labelled(
                        ui,
                        label,
                        SheetGrid::LABEL_WIDTH,
                        SheetGrid::COLUMN_GAP,
                        vec2(600.0, CoverageBadge::HEIGHT),
                        |ui, _| {
                            ui.spacing_mut().item_spacing = vec2(<spacing::S as SpacingToken>::VALUE, 0.0);
                            for badge in badges {
                                CoverageBadge::new(status, Self::words(badge)).show(ui);
                            }
                        },
                    );
                }
            });
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::COLUMN, &["RIGHT", "ABOVE", "BELOW"]);
                let placements = [BadgePlacement::Right, BadgePlacement::Above, BadgePlacement::Below];
                SheetGrid::row(ui, "Placement", Self::COLUMN, Self::CELL_HEIGHT, 3, |ui, index, cell| {
                    Self::placed(ui, cell, placements[index]);
                });
            });
        });
    }

    fn words(text: &str) -> Text {
        Text::user(UserText::from_model(text.to_owned()))
    }

    // a stand-in "Duplicate scene" control with a planned badge placed against it
    fn placed(ui: &mut Ui, cell: Rect, placement: BadgePlacement) {
        let style = TextStyle::new(TypeSize::Help, FontFace::Semibold);
        let label = Self::words("Duplicate scene");
        let padding = <spacing::ButtonX as SpacingToken>::VALUE;
        let height = <spacing::Field as SpacingToken>::VALUE;
        let width = Draw::measure(ui.ctx(), "Duplicate scene", style).x + padding * 2.0;
        let control = Align2::LEFT_CENTER.align_size_within_rect(vec2(width, height), cell);
        ui.allocate_rect(control, Sense::hover());
        let painter = ui.painter();
        Draw::fill(painter, control, Corner::Control, <color::BgField as ColorToken>::VALUE);
        let ink = <color::Text as ColorToken>::VALUE;
        Draw::text(painter, control.center(), Align2::CENTER_CENTER, &label, style, ink, TextFit::Line);
        PlacedBadge::new(StatusColor::Planned, Self::words("G11 PLANNED"), placement).paint_on(painter, control);
    }
}
