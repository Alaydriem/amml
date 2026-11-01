use std::f32::consts::FRAC_PI_4;

use amml_i18n::text::{Text, UserText};
use amml_ui::draw::{Corner, Draw};
use amml_ui::gallery::SheetGrid;
use amml_ui::glyph::Glyph;
use amml_ui::tokens::{ColorToken, color};
use egui::{Color32, Rect, Sense, Ui, pos2, vec2};

use crate::gallery::GalleryState;
use crate::glyph::{
    BookmarkFlag, CameraSegment, CheckpointTick, EditLockPill, NameTag, PathMapCone, Playhead, SceneCameraFrustum,
};
use crate::tokens::color as studio;

/// Paper: Assets, `Group / Drawn glyphs` (`307-0`): `Glyph / playhead` to `Glyph / edit lock pill`. Each glyph in
/// a 248 × 130 panel under its caption, four to a row. The key glyphs are on the amml-ui "Key glyphs" sheet.
pub struct GlyphsPage;

impl GlyphsPage {
    /// Paper: each glyph artboard's preview panel.
    const CELL: [f32; 2] = [248.0, 130.0];
    const PER_ROW: usize = 4;
    /// Paper: `Glyph / checkpoint tick`, the box of each tick sample.
    const TICK_BOX: egui::Vec2 = vec2(20.0, 20.0);
    /// Paper: `Glyph / scene camera frustum`, the box of each frustum sample.
    const FRUSTUM_BOX: egui::Vec2 = vec2(60.0, 56.0);
    /// Paper: `Glyph / player name tag`, how far the first tag sits above the cell centre.
    const TAG_RISE: egui::Vec2 = vec2(0.0, 14.0);
    const TITLES: [&'static str; 8] = [
        "PLAYHEAD",
        "BOOKMARK FLAG",
        "CHECKPOINT TICK",
        "SCENE CAMERA FRUSTUM",
        "PATH MAP CAMERA CONE",
        "PLAYER NAME TAG",
        "CAMERA SEGMENT + EASING PILL",
        "EDIT LOCK PILL",
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Glyphs", |ui| {
            for row in Self::TITLES.chunks(Self::PER_ROW).enumerate() {
                let (first, titles) = row;
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(SheetGrid::COLUMN_GAP, 0.0);
                    for (offset, title) in titles.iter().enumerate() {
                        SheetGrid::column(ui, title, Self::CELL[0], |ui| {
                            let (cell, _) = ui.allocate_exact_size(vec2(Self::CELL[0], Self::CELL[1]), Sense::hover());
                            Draw::fill(ui.painter(), cell, Corner::Panel, <color::BgPanel as ColorToken>::VALUE);
                            Self::glyph(ui, first * Self::PER_ROW + offset, cell);
                        });
                    }
                });
            }
        });
    }

    fn words(text: &str) -> Text {
        Text::user(UserText::from_model(text.to_owned()))
    }

    // the Paper samples of glyph `index`, spread across `cell`
    fn glyph(ui: &Ui, index: usize, cell: Rect) {
        let painter = ui.painter();
        let red = <color::Playhead as ColorToken>::VALUE;
        let amber = <studio::SceneCamera as ColorToken>::VALUE;
        let white = Color32::WHITE;
        let slots = |count: usize| {
            let width = cell.width() / count as f32;
            (0..count).map(move |slot| {
                Rect::from_min_size(pos2(cell.left() + width * slot as f32, cell.top()), vec2(width, cell.height()))
            })
        };
        match index {
            0 => {
                for (slot, colour) in slots(2).zip([red, amber]) {
                    let rect = Rect::from_center_size(slot.center(), vec2(Playhead::HEAD_WIDTH, 100.0));
                    Playhead { label: Self::words("32.95") }.paint(painter, rect, colour);
                }
            }
            1 => {
                for (slot, colour) in slots(2).zip([<studio::Marker as ColorToken>::VALUE, white]) {
                    let rect = Rect::from_center_size(slot.center(), vec2(BookmarkFlag::WIDTH, 90.0));
                    BookmarkFlag { label: Self::words("portal") }.paint(painter, rect, colour);
                }
            }
            2 => {
                let line = cell.center().y + 12.0;
                Draw::divider(
                    painter,
                    egui::Rangef::new(cell.left() + 24.0, cell.right() - 24.0),
                    line,
                    <color::Line as ColorToken>::VALUE,
                );
                for slot in slots(4) {
                    let rect = Rect::from_center_size(pos2(slot.center().x, line - 10.0), Self::TICK_BOX);
                    CheckpointTick.paint(painter, rect, <studio::Checkpoint as ColorToken>::VALUE);
                }
            }
            3 => {
                for (slot, (colour, filled)) in slots(3).zip([(amber, false), (white, true), (red, true)]) {
                    let rect = Rect::from_center_size(slot.center(), Self::FRUSTUM_BOX);
                    SceneCameraFrustum { filled }.paint(painter, rect, colour);
                }
            }
            4 => {
                for (slot, (colour, heading)) in slots(2).zip([(red, 0.0), (amber, FRAC_PI_4)]) {
                    let side = PathMapCone::REACH * 2.0;
                    PathMapCone { heading }.paint(
                        painter,
                        Rect::from_center_size(slot.center(), vec2(side, side)),
                        colour,
                    );
                }
            }
            5 => {
                let top = Rect::from_center_size(cell.center() - Self::TAG_RISE, vec2(cell.width(), 24.0));
                NameTag { name: Self::words("nether_nomad") }.paint(painter, top);
                for (slot, name) in slots(2).zip(["PixelPilgrim", "Kestrel"]) {
                    let rect =
                        Rect::from_center_size(pos2(slot.center().x, cell.center().y + 14.0), vec2(slot.width(), 24.0));
                    NameTag { name: Self::words(name) }.paint(painter, rect);
                }
            }
            6 => {
                for (row, (ease, hold)) in [("Smooth", false), ("Hold", true)].into_iter().enumerate() {
                    let y = cell.center().y - CameraSegment::HEIGHT + row as f32 * (CameraSegment::HEIGHT + 8.0);
                    let rect =
                        Rect::from_min_size(pos2(cell.center().x - 110.0, y), vec2(220.0, CameraSegment::HEIGHT));
                    CameraSegment { ease: Self::words(ease), hold }.paint(painter, rect);
                }
            }
            _ => {
                EditLockPill { reason: Self::words("Read only") }.paint(painter, cell);
            }
        }
    }
}
