use egui::{Rangef, Response, Ui};

use crate::draw::Draw;
use crate::tokens::{ColorToken, color};
use crate::widget::SettingRow;

/// The rows of a section card, added one at a time. A line goes between two rows, none after the
/// last.
pub struct SectionCardRows<'u> {
    ui: &'u mut Ui,
    count: usize,
}

impl<'u> SectionCardRows<'u> {
    pub(crate) fn new(ui: &'u mut Ui) -> Self {
        Self { ui, count: 0 }
    }

    pub fn row(&mut self, row: SettingRow<'_>) -> Response {
        if self.count > 0 {
            // inset like the rows' text, `--color-line`, on the edge between the two rows
            let rect = self.ui.max_rect();
            let inset = SettingRow::CARD_PAD_X;
            let x = Rangef::new(rect.left() + inset, rect.right() - inset);
            Draw::divider(self.ui.painter(), x, self.ui.cursor().top(), <color::Line as ColorToken>::VALUE);
        }
        self.count += 1;
        row.in_card().show(self.ui)
    }
}
