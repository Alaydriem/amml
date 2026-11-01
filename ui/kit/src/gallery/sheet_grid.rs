use egui::{Align, Frame, Layout, Rect, Sense, Ui, UiBuilder, Vec2, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::tokens::{ColorToken, color};

/// The chrome every Paper widget sheet shares: the page, the title, the state panel with small-caps
/// column heads and a row-label column. The numbers are the reference sheets' own layout, not tokens.
pub struct SheetGrid;

impl SheetGrid {
    pub const PAGE_PADDING: f32 = 40.0;
    pub const SECTION_GAP: f32 = 32.0;
    pub const PANEL_PADDING: f32 = 24.0;
    pub const ROW_GAP: f32 = 16.0;
    pub const COLUMN_GAP: f32 = 16.0;
    pub const HEAD_GAP: f32 = 12.0;
    pub const LABEL_WIDTH: f32 = 120.0;
    /// The "IN A SETTING ROW" and "IN A LIST" example panels.
    pub const LIST_WIDTH: f32 = 388.0;
    pub const LIST_PADDING: f32 = 8.0;

    /// The sheet page: `--color-bg-app`, padding 40, the title, then the sections 32 apart.
    pub fn page(ui: &mut Ui, title: &str, body: impl FnOnce(&mut Ui)) {
        let width = super::GallerySheet::WIDTH - Self::PAGE_PADDING * 2.0;
        // the page is the whole artboard, so the host panel's margin does not show
        ui.painter().rect_filled(ui.ctx().content_rect(), 0, <color::BgApp as ColorToken>::VALUE);
        Frame::new()
            .fill(<color::BgApp as ColorToken>::VALUE)
            .inner_margin(Draw::margin(Self::PAGE_PADDING, Self::PAGE_PADDING))
            .show(ui, |ui| {
                ui.set_width(width);
                ui.spacing_mut().item_spacing = vec2(0.0, Self::SECTION_GAP);
                let style = TextStyle::new(TypeSize::Heading, FontFace::Semibold);
                Self::label(ui, title, style, <color::Text as ColorToken>::VALUE);
                body(ui);
            });
    }

    /// A state panel: `--color-bg-panel`, 1 px `--color-line`, `--radius-panel`, padding 24, rows 16 apart.
    pub fn panel(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
        Self::panel_with_gap(ui, Self::ROW_GAP, body);
    }

    /// A state panel whose rows are `gap` apart.
    pub fn panel_with_gap(ui: &mut Ui, gap: f32, body: impl FnOnce(&mut Ui)) {
        Self::frame(Self::PANEL_PADDING).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing = vec2(0.0, gap);
            body(ui);
        });
    }

    /// The small example panel under a sheet: 388 wide, padding 8, rows touching.
    pub fn list_panel(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
        Self::frame(Self::LIST_PADDING).show(ui, |ui| {
            ui.set_width(Self::LIST_WIDTH - Self::LIST_PADDING * 2.0);
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            body(ui);
        });
    }

    /// A small-caps section head above loose examples, 12 above its content.
    pub fn section(ui: &mut Ui, head: &str, body: impl FnOnce(&mut Ui)) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, Self::HEAD_GAP);
            Self::label(ui, head, Self::head_style(), <color::TextFaint as ColorToken>::VALUE);
            body(ui);
        });
    }

    /// A small-caps head, such as the one at the top of a panel.
    pub fn head(ui: &mut Ui, head: &str) {
        Self::label(ui, head, Self::head_style(), <color::TextFaint as ColorToken>::VALUE);
    }

    /// A column `width` wide with a small-caps head 12 above its content (Empty state, Loading state).
    pub fn column(ui: &mut Ui, head: &str, width: f32, body: impl FnOnce(&mut Ui)) {
        ui.vertical(|ui| {
            ui.set_width(width);
            Self::section(ui, head, body);
        });
    }

    /// Sample text that wraps at the width left in `ui`.
    pub fn paragraph(ui: &mut Ui, text: &str, style: TextStyle, color: egui::Color32) {
        let galley = Draw::galley(ui.painter(), text, style, color, TextFit::Wrap(ui.available_width()));
        let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
        ui.painter().galley(rect.min, galley, color);
    }

    /// The stand-in for the game picture under cards, pills and modals: a flat `--color-bg-app`
    /// block with `--radius-panel`. The studio bundles no images.
    pub fn picture(ui: &Ui, rect: Rect) {
        Draw::fill(ui.painter(), rect, Corner::Panel, <color::BgApp as ColorToken>::VALUE);
    }

    /// The column heads: an empty label slot, then one head per column, 16 apart.
    pub fn heads(ui: &mut Ui, column: f32, heads: &[&str]) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(Self::COLUMN_GAP, 0.0);
            ui.allocate_exact_size(vec2(Self::LABEL_WIDTH, 0.0), Sense::hover());
            for head in heads {
                let size = vec2(column, Self::head_style().size.line_height());
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                Self::paint_label(ui, rect, head, Self::head_style(), <color::TextFaint as ColorToken>::VALUE);
            }
        });
    }

    /// A row: its label, then `count` cells of `column` x `height`, 16 apart. `cell` gets each
    /// cell's index and rect.
    pub fn row(
        ui: &mut Ui,
        label: &str,
        column: f32,
        height: f32,
        count: usize,
        mut cell: impl FnMut(&mut Ui, usize, Rect),
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(Self::COLUMN_GAP, 0.0);
            Self::row_label(ui, label, Self::LABEL_WIDTH, height);
            for index in 0..count {
                let (rect, _) = ui.allocate_exact_size(vec2(column, height), Sense::hover());
                let mut child =
                    ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)));
                cell(&mut child, index, rect);
            }
        });
    }

    /// A row like `row`, whose label is a small-caps head (the Menu item sheet's rows).
    pub fn head_row(
        ui: &mut Ui,
        label: &str,
        column: f32,
        height: f32,
        count: usize,
        mut cell: impl FnMut(&mut Ui, usize, Rect),
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(Self::COLUMN_GAP, 0.0);
            let (rect, _) = ui.allocate_exact_size(vec2(Self::LABEL_WIDTH, height), Sense::hover());
            Self::paint_label(ui, rect, label, Self::head_style(), <color::TextFaint as ColorToken>::VALUE);
            for index in 0..count {
                let (rect, _) = ui.allocate_exact_size(vec2(column, height), Sense::hover());
                let mut child =
                    ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)));
                cell(&mut child, index, rect);
            }
        });
    }

    /// A row with a `label_width` label, `gap`, then one `width` x `height` area for `body`.
    pub fn labelled(
        ui: &mut Ui,
        label: &str,
        label_width: f32,
        gap: f32,
        size: Vec2,
        body: impl FnOnce(&mut Ui, Rect),
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, 0.0);
            Self::row_label(ui, label, label_width, size.y);
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)));
            body(&mut child, rect);
        });
    }

    /// A rect of `size` at the left of `cell`, centred on its height.
    pub fn place(cell: Rect, size: Vec2) -> Rect {
        Rect::from_min_size(pos2(cell.left(), cell.center().y - size.y / 2.0), size)
    }

    fn frame(padding: f32) -> Frame {
        Frame::new()
            .fill(<color::BgPanel as ColorToken>::VALUE)
            .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
            .corner_radius(Corner::Panel.radius(Rect::NOTHING))
            .inner_margin(Draw::margin(padding, padding))
    }

    fn row_label(ui: &mut Ui, label: &str, width: f32, height: f32) {
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let style = TextStyle::new(TypeSize::Note, FontFace::Regular);
        Self::paint_label(ui, rect, label, style, <color::TextDim as ColorToken>::VALUE);
    }

    /// One line of sheet text at its own size.
    pub fn label(ui: &mut Ui, text: &str, style: TextStyle, color: egui::Color32) {
        let galley = Draw::galley(ui.painter(), text, style, color, TextFit::Line);
        let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
        ui.painter().galley(rect.min, galley, color);
    }

    fn paint_label(ui: &Ui, rect: Rect, text: &str, style: TextStyle, color: egui::Color32) {
        let galley = Draw::galley(ui.painter(), text, style, color, TextFit::Truncate(rect.width()));
        let at = pos2(rect.left(), rect.center().y - galley.size().y / 2.0);
        ui.painter().galley(at, galley, color);
    }

    /// The small-caps head style: caption, semibold, `--tracking-section`.
    pub fn head_style() -> TextStyle {
        TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked()
    }
}
