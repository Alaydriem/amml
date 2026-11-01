use egui::{Color32, CursorIcon, Id, Painter, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::draw::{Corner, Draw};
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Swatch row`. A row of colour squares; the selected one has a white outline set off
/// from it. The caller names the colours (scene colours in amml-studio-ui).
pub struct SwatchRow<'a> {
    id: Id,
    selected: &'a mut usize,
    colours: Vec<Color32>,
}

impl<'a> SwatchRow<'a> {
    /// Paper: Swatch row sheet, the side of one square.
    pub const SIDE: f32 = 22.0;
    /// Paper: Swatch row sheet, the row height.
    pub const HEIGHT: f32 = 30.0;
    /// Paper: Swatch row sheet, the room left of the first square for the outline.
    const LEAD: f32 = 3.0;
    /// Paper: Swatch row sheet, the gap between the square and its outline.
    const OUTLINE_OFFSET: f32 = 3.0;

    pub fn new(id: Id, selected: &'a mut usize, colours: Vec<Color32>) -> Self {
        Self { id, selected, colours }
    }

    /// The square of swatch `index` in a row painted at `row`.
    pub fn cell(row: Rect, index: usize) -> Rect {
        let step = Self::SIDE + <spacing::S as SpacingToken>::VALUE;
        let left = row.left() + Self::LEAD + step * index as f32;
        Rect::from_min_size(pos2(left, row.center().y - Self::SIDE / 2.0), vec2(Self::SIDE, Self::SIDE))
    }

    /// The width the row takes.
    pub fn width(&self) -> f32 {
        let count = self.colours.len() as f32;
        Self::LEAD + count * Self::SIDE + (count - 1.0).max(0.0) * <spacing::S as SpacingToken>::VALUE
    }

    /// Shows the row; the response is changed when another swatch was picked.
    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, mut response) = ui.allocate_exact_size(vec2(self.width(), Self::HEIGHT), Sense::hover());
        for index in 0..self.colours.len() {
            let cell = Self::cell(rect, index);
            let swatch = ui.interact(cell, self.id.with(index), Sense::click());
            if swatch.clicked() && *self.selected != index {
                *self.selected = index;
                response.mark_changed();
            }
            if swatch.has_focus() {
                FocusRing::paint(ui.painter(), cell, Corner::Control.radius(cell));
            }
            swatch.on_hover_cursor(CursorIcon::PointingHand);
        }
        self.paint(ui.painter(), rect);
        response
    }

    /// Paints the row at `rect`.
    pub fn paint(&self, painter: &Painter, rect: Rect) {
        for (index, colour) in self.colours.iter().enumerate() {
            let cell = Self::cell(rect, index);
            Draw::fill(painter, cell, Corner::Control, *colour);
            if index == *self.selected {
                let outline = cell.expand(Self::OUTLINE_OFFSET);
                let stroke =
                    Stroke::new(<spacing::BorderStrong as SpacingToken>::VALUE, <color::OnAccent as ColorToken>::VALUE);
                let radius = Corner::Control.radius(cell);
                let outer = egui::CornerRadius::same(radius.nw.saturating_add(Self::OUTLINE_OFFSET as u8));
                painter.rect_stroke(outline, outer, stroke, StrokeKind::Outside);
            }
        }
    }
}
