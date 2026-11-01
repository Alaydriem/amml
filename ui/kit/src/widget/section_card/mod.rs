mod section_card_rows;

pub use section_card_rows::SectionCardRows;

use amml_i18n::text::Text;
use egui::{Frame, Rect, Response, Ui, Vec2};

use crate::draw::{Corner, Draw};
use crate::tokens::{ColorToken, color};
use crate::widget::SectionHeader;

/// Paper: Mod manager, `Widget / Section card`. A section header over a panel card of setting
/// rows, with a line between rows. The rows take the card's look: a `--text-body` label, help
/// under it, and the card's padding. It takes the full width.
pub struct SectionCard {
    title: Text,
}

impl SectionCard {
    pub fn new(title: Text) -> Self {
        Self { title }
    }

    /// Shows the header and the card; `rows` adds the rows. Returns the card's response.
    pub fn show(self, ui: &mut Ui, rows: impl FnOnce(&mut SectionCardRows)) -> Response {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            SectionHeader::new(self.title).show(ui);
            Frame::new()
                .fill(<color::BgPanel as ColorToken>::VALUE)
                .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
                .corner_radius(Corner::Panel.radius(Rect::NOTHING))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    rows(&mut SectionCardRows::new(ui));
                })
                .response
        })
        .inner
    }
}
