mod carousel_item;
mod carousel_mark;

pub use carousel_item::CarouselItem;
pub use carousel_mark::CarouselMark;

use egui::{Align2, Color32, Painter, Pos2, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::{Icon, IconShape, IconSize};
use crate::tokens::rule::{Elevation, SoftFill};
use crate::tokens::{ColorToken, color};

/// Paper: Mod manager, `Widget / Carousel`. The current item in the middle with its mark, name and value, the
/// previous and next items dimmed at the sides (wrapping at the ends), and one position dot per item. Display
/// only: keys drive it, so it takes no clicks.
pub struct Carousel<'a> {
    items: &'a [CarouselItem],
    current: usize,
}

impl<'a> Carousel<'a> {
    /// Paper: Carousel sheet, the padding above the row.
    const PAD_TOP: f32 = 14.0;
    /// Paper: Carousel sheet, the padding at the left and right.
    const PAD_SIDE: f32 = 16.0;
    /// Paper: Carousel sheet, the padding under the dots.
    const PAD_BOTTOM: f32 = 12.0;
    /// Paper: Carousel sheet, the gap between the row and the dots.
    const DOTS_GAP: f32 = 10.0;
    /// Paper: Carousel sheet, the gap between a side slot and the current item.
    const ROW_GAP: f32 = 8.0;
    /// Paper: Carousel sheet, the side of a previous or next slot.
    const SLOT: f32 = 36.0;
    /// Paper: Carousel sheet, how much of a side slot shows through.
    const SLOT_OPACITY: f32 = 0.5;
    /// Paper: Carousel sheet, the padding above and under the current item.
    const ITEM_PAD_Y: f32 = 8.0;
    /// Paper: Carousel sheet, the padding at the left and right of the current item.
    const ITEM_PAD_X: f32 = 12.0;
    /// Paper: Carousel sheet, the gap between the mark tile, the text column and the change hint.
    const ITEM_GAP: f32 = 12.0;
    /// Paper: Carousel sheet, the side of the mark tile.
    const TILE: f32 = 40.0;
    /// Paper: Carousel sheet, the gap between the name and the value.
    const TEXT_GAP: f32 = 2.0;
    /// Paper: Carousel sheet, the line height of each arrow of the change hint.
    const HINT_LINE: f32 = 14.0;
    /// Paper: Carousel sheet, the side of a position dot, and the height of the current one.
    const DOT: f32 = 5.0;
    /// Paper: Carousel sheet, the width of the current item's dot.
    const DOT_CURRENT: f32 = 14.0;
    /// Paper: Carousel sheet, the gap between dots.
    const DOT_GAP: f32 = 4.0;
    const SLOT_LETTER: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Semibold);
    const TILE_LETTER: TextStyle = TextStyle::new(TypeSize::Title, FontFace::Semibold);
    const NAME: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Semibold);
    const VALUE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);

    pub fn new(items: &'a [CarouselItem], current: usize) -> Self {
        Self { items, current }
    }

    /// Shows the carousel as wide as the ui allows.
    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), self.height()), Sense::hover());
        self.paint(ui.painter(), rect.min, rect.width());
        response
    }

    /// Paints the carousel with its top left at `top_left`, `width` wide. Returns its rect.
    pub fn paint(&self, painter: &Painter, top_left: Pos2, width: f32) -> Rect {
        let rect = Rect::from_min_size(top_left, vec2(width, self.height()));
        let count = self.items.len();
        if count == 0 {
            return rect;
        }
        let current = self.current % count;
        let row = Rect::from_min_size(
            pos2(rect.left() + Self::PAD_SIDE, rect.top() + Self::PAD_TOP),
            vec2((width - 2.0 * Self::PAD_SIDE).max(0.0), Self::row_height()),
        );
        let slot_y = row.center().y - Self::SLOT / 2.0;
        let previous = Rect::from_min_size(pos2(row.left(), slot_y), vec2(Self::SLOT, Self::SLOT));
        let next = Rect::from_min_size(pos2(row.right() - Self::SLOT, slot_y), vec2(Self::SLOT, Self::SLOT));
        let item = Rect::from_min_max(
            pos2(previous.right() + Self::ROW_GAP, row.top()),
            pos2(next.left() - Self::ROW_GAP, row.bottom()),
        );
        Self::paint_slot(painter, previous, &self.items[(current + count - 1) % count].mark);
        Self::paint_slot(painter, next, &self.items[(current + 1) % count].mark);
        Self::paint_current(painter, item, &self.items[current]);
        Self::paint_dots(painter, rect, row.bottom() + Self::DOTS_GAP, count, current);
        rect
    }

    /// The height the carousel takes.
    pub fn height(&self) -> f32 {
        Self::PAD_TOP + Self::row_height() + Self::DOTS_GAP + Self::DOT + Self::PAD_BOTTOM
    }

    fn row_height() -> f32 {
        Self::TILE + 2.0 * Self::ITEM_PAD_Y
    }

    fn paint_slot(painter: &Painter, rect: Rect, mark: &CarouselMark) {
        let fade = Self::SLOT_OPACITY;
        Draw::fill(painter, rect, Corner::Control, <color::BgField as ColorToken>::VALUE.gamma_multiply(fade));
        match mark {
            CarouselMark::Icon(icon) => {
                let dim = <color::TextDim as ColorToken>::VALUE.gamma_multiply(fade);
                icon.paint(painter, rect, IconSize::Regular, dim);
            }
            CarouselMark::Letter(letter, tone) => {
                let colour = tone.gamma_multiply(fade);
                Draw::text(
                    painter,
                    rect.center(),
                    Align2::CENTER_CENTER,
                    letter,
                    Self::SLOT_LETTER,
                    colour,
                    TextFit::Line,
                );
            }
        }
    }

    fn paint_current(painter: &Painter, rect: Rect, item: &CarouselItem) {
        Draw::surface(
            painter,
            rect,
            Corner::Panel,
            <color::BgActive as ColorToken>::VALUE,
            Some(<color::LineStrong as ColorToken>::VALUE),
            Elevation::Flat,
        );
        let tile = Rect::from_min_size(
            pos2(rect.left() + Self::ITEM_PAD_X, rect.center().y - Self::TILE / 2.0),
            vec2(Self::TILE, Self::TILE),
        );
        match &item.mark {
            CarouselMark::Icon(icon) => {
                Draw::fill(painter, tile, Corner::Control, <color::AccentSoft as ColorToken>::VALUE);
                icon.paint(painter, tile, IconSize::Regular, <color::Accent as ColorToken>::VALUE);
            }
            CarouselMark::Letter(letter, tone) => {
                Draw::fill(painter, tile, Corner::Control, SoftFill::of(*tone));
                let centre = tile.center();
                Draw::text(painter, centre, Align2::CENTER_CENTER, letter, Self::TILE_LETTER, *tone, TextFit::Line);
            }
        }
        let hint_left = rect.right() - Self::ITEM_PAD_X - Self::HINT_LINE;
        let text_left = tile.right() + Self::ITEM_GAP;
        let room = (hint_left - Self::ITEM_GAP - text_left).max(0.0);
        let name_height = Self::NAME.size.line_height();
        let value_height = Self::VALUE.size.line_height();
        let top = rect.center().y - (name_height + Self::TEXT_GAP + value_height) / 2.0;
        let name = <color::Text as ColorToken>::VALUE;
        Draw::text(
            painter,
            pos2(text_left, top),
            Align2::LEFT_TOP,
            &item.name,
            Self::NAME,
            name,
            TextFit::Truncate(room),
        );
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let value_at = pos2(text_left, top + name_height + Self::TEXT_GAP);
        Draw::text(painter, value_at, Align2::LEFT_TOP, &item.value, Self::VALUE, faint, TextFit::Truncate(room));
        let hint_top = rect.center().y - Self::HINT_LINE;
        for (index, icon) in [Icon::ChevronUp, Icon::ChevronDown].into_iter().enumerate() {
            let cell = Rect::from_min_size(
                pos2(hint_left, hint_top + index as f32 * Self::HINT_LINE),
                vec2(Self::HINT_LINE, Self::HINT_LINE),
            );
            Self::paint_hint(painter, cell, icon);
        }
    }

    // one arrow of the change hint, at the caption size in the dim colour
    fn paint_hint(painter: &Painter, cell: Rect, icon: Icon) {
        let colour = <color::TextDim as ColorToken>::VALUE;
        match icon.shape() {
            IconShape::Font(code) => {
                let font = FontFace::Icons.id(TypeSize::Caption.points());
                painter.text(cell.center(), Align2::CENTER_CENTER, code, font, colour);
            }
            IconShape::Drawn(_) => icon.paint(painter, cell, IconSize::Small, colour),
        }
    }

    fn paint_dots(painter: &Painter, rect: Rect, top: f32, count: usize, current: usize) {
        let total = (count as f32 - 1.0) * (Self::DOT + Self::DOT_GAP) + Self::DOT_CURRENT;
        let mut x = rect.center().x - total / 2.0;
        for index in 0..count {
            let (width, fill): (f32, Color32) = if index == current {
                (Self::DOT_CURRENT, <color::Accent as ColorToken>::VALUE)
            } else {
                (Self::DOT, <color::LineStrong as ColorToken>::VALUE)
            };
            let dot = Rect::from_min_size(pos2(x, top), vec2(width, Self::DOT));
            Draw::fill(painter, dot, Corner::Full, fill);
            x += width + Self::DOT_GAP;
        }
    }
}
