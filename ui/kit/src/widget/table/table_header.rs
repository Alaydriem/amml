use amml_i18n::text::Text;
use egui::{Rangef, Response, Sense, Ui, vec2};

use super::{TableAlign, TableColumn, TableLayout};
use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Mod manager, `Widget / Table header + Table row`, the header. One small-caps head per
/// column over a line. It takes the full width.
pub struct TableHeader {
    columns: Vec<(Text, TableColumn)>,
    aligns: Vec<TableAlign>,
}

impl TableHeader {
    /// Paper: Table, the header height.
    const HEIGHT: f32 = 36.0;
    const HEAD: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();

    pub fn new(columns: Vec<(Text, TableColumn)>) -> Self {
        Self { columns, aligns: Vec::new() }
    }

    /// Each column's alignment, in column order; a column without one starts at the start edge.
    pub fn aligns(mut self, aligns: &[TableAlign]) -> Self {
        self.aligns = aligns.to_vec();
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::HEIGHT), Sense::hover());
        let painter = ui.painter();
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let columns: Vec<TableColumn> = self.columns.iter().map(|(_, column)| *column).collect();
        for (index, ((head, _), cell)) in self.columns.iter().zip(TableLayout::cells(&columns, rect)).enumerate() {
            let align = self.aligns.get(index).copied().unwrap_or_default();
            let (at, anchor) = align.anchor(cell);
            Draw::text(painter, at, anchor, head, Self::HEAD, faint, TextFit::Truncate(cell.width()));
        }
        let line = <color::Line as ColorToken>::VALUE;
        Draw::divider(
            painter,
            Rangef::new(rect.left(), rect.right()),
            rect.bottom() - <spacing::Divider as SpacingToken>::VALUE,
            line,
        );
        response
    }
}
