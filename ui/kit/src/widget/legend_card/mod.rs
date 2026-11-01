mod legend_row;

pub use legend_row::LegendRow;

use amml_i18n::text::Text;
use egui::{Painter, Pos2, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::text::UiText;
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Legend card`. A card laid over a picture: a title, an optional note and one row per
/// colour the picture uses, each a rounded square swatch and its meaning.
pub struct LegendCard {
    title: Text,
    note: Option<Text>,
    rows: Vec<LegendRow>,
}

impl LegendCard {
    /// Paper: Legend card sheet, how much of the app background shows through.
    const FADE: f32 = 0.84;
    /// Paper: Legend card sheet, the padding above the title.
    const PAD_TOP: f32 = 14.0;
    /// Paper: Legend card sheet, the padding under the last row.
    const PAD_BOTTOM: f32 = 16.0;
    /// Paper: Legend card sheet, the padding at the left and right.
    const PAD_SIDE: f32 = 18.0;
    /// Paper: Legend card sheet, the swatch side.
    const SWATCH: f32 = 12.0;
    /// Paper: Legend card sheet, the gap between rows and between a swatch and its words.
    const ROW_GAP: f32 = 10.0;
    const TITLE: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);
    const NOTE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);
    const LABEL: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);

    pub fn new(title: Text) -> Self {
        Self { title, note: None, rows: Vec::new() }
    }

    pub fn note(mut self, note: Text) -> Self {
        self.note = Some(note);
        self
    }

    pub fn rows(mut self, rows: Vec<LegendRow>) -> Self {
        self.rows = rows;
        self
    }

    /// Shows the card `width` wide.
    pub fn show(&self, ui: &mut Ui, width: f32) -> Response {
        let height = self.height();
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        self.paint(ui.painter(), rect.min, width);
        response
    }

    /// Paints the card with its top left at `top_left`. Returns its rect.
    pub fn paint(&self, painter: &Painter, top_left: Pos2, width: f32) -> Rect {
        let rect = Rect::from_min_size(top_left, vec2(width, self.height()));
        let fill = <color::BgApp as ColorToken>::VALUE.gamma_multiply(Self::FADE);
        Draw::surface(
            painter,
            rect,
            Corner::Panel,
            fill,
            Some(<color::LineOverlay as ColorToken>::VALUE),
            Elevation::Flat,
        );
        let border = <spacing::Border as SpacingToken>::VALUE;
        let left = rect.left() + border + Self::PAD_SIDE;
        let room = (rect.width() - 2.0 * (border + Self::PAD_SIDE)).max(0.0);
        let mut y = rect.top() + border + Self::PAD_TOP;
        y += Self::line(painter, &self.title, pos2(left, y), Self::TITLE, <color::Text as ColorToken>::VALUE, room);
        let dim = <color::TextDim as ColorToken>::VALUE;
        if let Some(note) = &self.note {
            y += <spacing::S as SpacingToken>::VALUE;
            y += Self::line(painter, note, pos2(left, y), Self::NOTE, dim, room);
        }
        y += <spacing::L as SpacingToken>::VALUE;
        for (index, row) in self.rows.iter().enumerate() {
            if index > 0 {
                y += Self::ROW_GAP;
            }
            let height = Self::LABEL.size.line_height();
            let swatch = Rect::from_center_size(
                pos2(left + Self::SWATCH / 2.0, y + height / 2.0),
                vec2(Self::SWATCH, Self::SWATCH),
            );
            Draw::fill(painter, swatch, Corner::Small, row.colour());
            let text_left = swatch.right() + Self::ROW_GAP;
            Self::line(painter, row.label(), pos2(text_left, y), Self::LABEL, dim, (left + room - text_left).max(0.0));
            y += height;
        }
        rect
    }

    /// The height the card takes for its note and rows.
    pub fn height(&self) -> f32 {
        let border = <spacing::Border as SpacingToken>::VALUE;
        let mut height =
            2.0 * border + Self::PAD_TOP + Self::TITLE.size.line_height() + <spacing::L as SpacingToken>::VALUE;
        if self.note.is_some() {
            height += <spacing::S as SpacingToken>::VALUE + Self::NOTE.size.line_height();
        }
        let count = self.rows.len() as f32;
        height += count * Self::LABEL.size.line_height() + (count - 1.0).max(0.0) * Self::ROW_GAP;
        height + Self::PAD_BOTTOM
    }

    // paints one line of text at `at` and returns its height
    fn line(painter: &Painter, text: &Text, at: Pos2, style: TextStyle, colour: egui::Color32, room: f32) -> f32 {
        let resolved = UiText::resolve(painter.ctx(), text);
        let galley = Draw::galley(painter, &resolved, style, colour, TextFit::Truncate(room));
        let height = style.size.line_height();
        painter.galley(at, galley, colour);
        height
    }
}
