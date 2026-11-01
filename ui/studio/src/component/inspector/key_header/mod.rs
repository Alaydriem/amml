mod key_header_tone;
mod key_header_view;

pub use key_header_tone::KeyHeaderTone;
pub use key_header_view::KeyHeaderView;

use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Rect, Response, Sense, Ui, pos2, vec2};

/// Paper: Editor components, `Editor / Key header` (`GAI-0`). The card at the top of the Key tab: a stripe and a dot
/// in the track colour, the title and its detail, and a chip at the right. It takes the width left.
pub struct KeyHeader<'a> {
    view: &'a KeyHeaderView,
}

impl<'a> KeyHeader<'a> {
    /// Paper: Key header sheet, the card height.
    const HEIGHT: f32 = 66.0;
    /// Paper: Key header sheet, the stripe size.
    const STRIPE: egui::Vec2 = vec2(3.0, 38.0);
    /// Paper: Key header sheet, the dot size, and how far its top is under the card's top.
    const DOT: f32 = 9.0;
    const DOT_TOP: f32 = 18.0;
    /// Paper: Key header sheet, the gap between the parts, and the room right of the chip.
    const GAP: f32 = 12.0;
    const PADDING_RIGHT: f32 = 14.0;
    /// Paper: Key header sheet, the gap between the title and its detail.
    const LINE_GAP: f32 = 6.0;
    /// Paper: Key header sheet, how far the chip's top is under the card's top, and its padding.
    const CHIP_TOP: f32 = 11.0;
    const CHIP_PADDING_X: f32 = 8.0;
    const CHIP_PADDING_Y: f32 = 3.0;
    const TITLE: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Semibold);
    const DETAIL: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);
    const CHIP: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);

    pub fn new(view: &'a KeyHeaderView) -> Self {
        Self { view }
    }

    pub fn show(&self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::HEIGHT), Sense::hover());
        let painter = ui.painter();
        let border = <spacing::Border as SpacingToken>::VALUE;
        let fill = <color::BgRaised as ColorToken>::VALUE;
        Draw::surface(painter, rect, Corner::Panel, fill, Some(<color::Line as ColorToken>::VALUE), Elevation::Flat);
        let tone = self.view.tone.color();
        let inside = rect.shrink(border);
        let stripe = Rect::from_center_size(pos2(inside.left() + Self::STRIPE.x / 2.0, rect.center().y), Self::STRIPE);
        Draw::fill(painter, stripe, Corner::Small, tone);
        let dot_centre =
            pos2(stripe.right() + Self::GAP + Self::DOT / 2.0, inside.top() + Self::DOT_TOP + Self::DOT / 2.0);
        painter.circle_filled(dot_centre, Self::DOT / 2.0, tone);

        let chip_text = UiText::resolve(ui.ctx(), &self.view.chip);
        let dim = <color::TextDim as ColorToken>::VALUE;
        let chip = Draw::galley(painter, &chip_text, Self::CHIP, dim, TextFit::Line);
        let chip_size = chip.size() + vec2(Self::CHIP_PADDING_X, Self::CHIP_PADDING_Y) * 2.0;
        let chip_rect = Rect::from_min_size(
            pos2(inside.right() - Self::PADDING_RIGHT - chip_size.x, inside.top() + Self::CHIP_TOP),
            chip_size,
        );
        Draw::fill(painter, chip_rect, Corner::Full, <color::BgField as ColorToken>::VALUE);
        painter.galley(chip_rect.min + vec2(Self::CHIP_PADDING_X, Self::CHIP_PADDING_Y), chip, dim);

        let left = dot_centre.x + Self::DOT / 2.0 + Self::GAP;
        let room = (chip_rect.left() - Self::GAP - left).max(0.0);
        let title = UiText::resolve(ui.ctx(), &self.view.title);
        let detail = UiText::resolve(ui.ctx(), &self.view.detail);
        let text = <color::Text as ColorToken>::VALUE;
        let title = Draw::galley(painter, &title, Self::TITLE, text, TextFit::Truncate(room));
        let detail = Draw::galley(painter, &detail, Self::DETAIL, dim, TextFit::Truncate(room));
        let top = rect.center().y - (title.size().y + Self::LINE_GAP + detail.size().y) / 2.0;
        let title_height = title.size().y;
        painter.galley(pos2(left, top), title, text);
        painter.galley(pos2(left, top + title_height + Self::LINE_GAP), detail, dim);
        response
    }
}
