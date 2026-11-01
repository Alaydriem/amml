use egui::{Align2, Frame, Margin, Rect, Sense, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::SectionHeader;

/// Paper: Widgets, `View part / Section header` (`6TG-0`). Three sections of an inspector column;
/// the scene swatch is an amml-ui stand-in for the studio-owned scene colour.
pub struct SectionHeaderSheet;

impl SectionHeaderSheet {
    /// Sheet chrome: `--container-inspector`, an amml-studio-ui token, so the sheet keeps the number.
    const INSPECTOR_WIDTH: f32 = 430.0;
    /// Paper: Section header, the scene swatch in the example field.
    const SWATCH: f32 = 10.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Section header", |ui| {
            Frame::new()
                .fill(<color::BgPanel as ColorToken>::VALUE)
                .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
                .corner_radius(Corner::Panel.radius(Rect::NOTHING))
                .show(ui, |ui| {
                    let border = <spacing::Border as SpacingToken>::VALUE;
                    ui.set_width(Self::INSPECTOR_WIDTH - border * 2.0);
                    ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                    Self::section(ui, "SCENE", false, Self::scene_field);
                    Self::section(ui, "RANGE", true, |ui| Self::value_row(ui, "Length", "8.60 s · 172 ticks"));
                    Self::section(ui, "CAMERA PATH", true, |ui| Self::value_row(ui, "Keys", "6"));
                });
        });
    }

    // one block: `--spacing-l` in from the sides and below, the header at its top
    fn section(ui: &mut Ui, title: &str, divider: bool, content: impl FnOnce(&mut Ui)) {
        let padding = Draw::margin(<spacing::L as SpacingToken>::VALUE, 0.0).left;
        Frame::new().inner_margin(Margin { left: padding, right: padding, top: 0, bottom: padding }).show(ui, |ui| {
            ui.set_width(ui.available_width());
            SectionHeader::new(Sample::text(title)).divider(divider).show(ui);
            content(ui);
        });
    }

    fn scene_field(ui: &mut Ui) {
        let (rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), <spacing::Field as SpacingToken>::VALUE), Sense::hover());
        Draw::fill(ui.painter(), rect, Corner::Control, <color::BgField as ColorToken>::VALUE);
        let left = rect.left() + <spacing::M as SpacingToken>::VALUE;
        let swatch =
            Rect::from_min_size(pos2(left, rect.center().y - Self::SWATCH / 2.0), vec2(Self::SWATCH, Self::SWATCH));
        Draw::fill(ui.painter(), swatch, Corner::Small, <color::Success as ColorToken>::VALUE);
        let text = <color::Text as ColorToken>::VALUE;
        let style = TextStyle::new(TypeSize::Label, FontFace::Regular);
        let at = pos2(swatch.right() + <spacing::S as SpacingToken>::VALUE, rect.center().y);
        Draw::text(ui.painter(), at, Align2::LEFT_CENTER, &Sample::text("Arrival"), style, text, TextFit::Line);
    }

    // a label on the left and its value on the right, `--spacing-row` high
    fn value_row(ui: &mut Ui, label: &str, value: &str) {
        let (rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), <spacing::Row as SpacingToken>::VALUE), Sense::hover());
        let dim = <color::TextDim as ColorToken>::VALUE;
        let help = TextStyle::new(TypeSize::Help, FontFace::Regular);
        Draw::text(
            ui.painter(),
            rect.left_center(),
            Align2::LEFT_CENTER,
            &Sample::text(label),
            help,
            dim,
            TextFit::Line,
        );
        let text = <color::Text as ColorToken>::VALUE;
        let note = TextStyle::new(TypeSize::Note, FontFace::Regular);
        Draw::text(
            ui.painter(),
            rect.right_center(),
            Align2::RIGHT_CENTER,
            &Sample::text(value),
            note,
            text,
            TextFit::Line,
        );
    }
}
