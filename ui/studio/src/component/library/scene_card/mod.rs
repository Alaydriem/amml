mod scene_card_view;

pub use scene_card_view::SceneCardView;

use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{CursorIcon, Rect, Response, Sense, Ui, pos2, vec2};

/// Paper: Editor components, `Editor / Scene card` (`G63-0`). A raised card with a stripe in the scene colour, the
/// scene's name and its range. It takes the width left; a click is reported through the response.
pub struct SceneCard<'a> {
    view: &'a SceneCardView,
}

impl<'a> SceneCard<'a> {
    /// Paper: Scene card sheet, the card height.
    pub const HEIGHT: f32 = 76.0;
    /// Paper: Scene card sheet, the room left and right of the parts.
    const PADDING: f32 = 14.0;
    /// Paper: Scene card sheet, the gap between the stripe and the words.
    const GAP: f32 = 11.0;
    /// Paper: Scene card sheet, the stripe size.
    const STRIPE: egui::Vec2 = vec2(3.0, 44.0);
    const NAME: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);
    const RANGE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);

    pub fn new(view: &'a SceneCardView) -> Self {
        Self { view }
    }

    pub fn show(&self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::HEIGHT), Sense::click());
        self.paint(ui, rect);
        response.on_hover_cursor(CursorIcon::PointingHand)
    }

    fn paint(&self, ui: &Ui, rect: Rect) {
        let painter = ui.painter();
        let fill = <color::BgRaised as ColorToken>::VALUE;
        Draw::surface(painter, rect, Corner::Panel, fill, Some(<color::Line as ColorToken>::VALUE), Elevation::Flat);
        let border = <spacing::Border as SpacingToken>::VALUE;
        let stripe = Rect::from_min_size(
            pos2(rect.left() + border + Self::PADDING, rect.center().y - Self::STRIPE.y / 2.0),
            Self::STRIPE,
        );
        Draw::fill(painter, stripe, Corner::Small, self.view.color.color());
        let left = stripe.right() + Self::GAP;
        let room = (rect.right() - border - Self::PADDING - left).max(0.0);
        let name = UiText::resolve(ui.ctx(), &self.view.name);
        let range = UiText::resolve(ui.ctx(), &self.view.range);
        let text = <color::Text as ColorToken>::VALUE;
        let dim = <color::TextDim as ColorToken>::VALUE;
        let name = Draw::galley(painter, &name, Self::NAME, text, TextFit::Truncate(room));
        let range = Draw::galley(painter, &range, Self::RANGE, dim, TextFit::Truncate(room));
        let gap = <spacing::S as SpacingToken>::VALUE;
        let top = rect.center().y - (name.size().y + gap + range.size().y) / 2.0;
        let name_height = name.size().y;
        painter.galley(pos2(left, top), name, text);
        painter.galley(pos2(left, top + name_height + gap), range, dim);
    }
}
