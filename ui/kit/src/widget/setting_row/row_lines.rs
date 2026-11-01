use std::sync::Arc;

use egui::{Galley, Painter, Pos2, Rect, pos2, vec2};

use crate::icon::{Icon, IconSize};
use crate::state::InteractionState;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::Chip;

/// A grown setting row's text column, laid out: the label with an optional chip after its first
/// line (or on its own line under the label when it has no room there), the help under it, and the
/// note with its icon under that.
pub(crate) struct RowLines {
    pub label: Arc<Galley>,
    pub chip: Option<Chip>,
    /// The chip goes on its own line under the label.
    pub chip_below: bool,
    /// The text column's width; the chip is cut to it.
    pub width: f32,
    pub help: Option<Arc<Galley>>,
    pub note: Option<Arc<Galley>>,
}

impl RowLines {
    /// Paper: Section card, the gap between the label, help and note lines.
    pub const LINE_GAP: f32 = 2.0;

    /// The indent of the note text: the icon and its gap.
    pub fn note_indent() -> f32 {
        IconSize::Small.points() + <spacing::Xs as SpacingToken>::VALUE
    }

    pub fn height(&self) -> f32 {
        let below = [&self.help, &self.note].into_iter().flatten().map(|galley| Self::LINE_GAP + galley.size().y);
        let chip = if self.chip_below && self.chip.is_some() { Self::LINE_GAP + Chip::height() } else { 0.0 };
        self.label.size().y + chip + below.sum::<f32>()
    }

    pub fn paint(self, painter: &Painter, top_left: Pos2) {
        let text = <color::Text as ColorToken>::VALUE;
        let first_row = self.label.rows.first().map_or(0.0, |row| row.rect().width());
        let first_centre = top_left.y + self.label.rows.first().map_or(0.0, |row| row.rect().center().y);
        let mut y = top_left.y + self.label.size().y;
        painter.galley(top_left, self.label, text);
        if let Some(chip) = self.chip {
            let room =
                if self.chip_below { self.width } else { self.width - first_row - <spacing::M as SpacingToken>::VALUE };
            let size = vec2(chip.desired_width(painter.ctx()).min(room.max(Chip::height())), Chip::height());
            let place = if self.chip_below {
                y += Self::LINE_GAP;
                let place = Rect::from_min_size(pos2(top_left.x, y), size);
                y += size.y;
                place
            } else {
                let x = top_left.x + first_row + <spacing::M as SpacingToken>::VALUE;
                Rect::from_min_size(pos2(x, first_centre - size.y / 2.0), size)
            };
            chip.paint(painter, place, InteractionState::Rest);
        }
        if let Some(help) = self.help {
            y += Self::LINE_GAP;
            let height = help.size().y;
            painter.galley(pos2(top_left.x, y), help, <color::TextDim as ColorToken>::VALUE);
            y += height;
        }
        if let Some(note) = self.note {
            y += Self::LINE_GAP;
            let warning = <color::Warning as ColorToken>::VALUE;
            let line = note.rows.first().map_or(0.0, |row| row.rect().height());
            let side = IconSize::Small.points();
            let slot = Rect::from_min_size(pos2(top_left.x, y + (line - side) / 2.0), vec2(side, side));
            Icon::Info.paint(painter, slot, IconSize::Small, warning);
            painter.galley(pos2(top_left.x + Self::note_indent(), y), note, warning);
        }
    }
}
