use egui::{Align2, Color32, Context, Painter, Rect, vec2};

use super::{KeyCapLabel, KeyCapSize};
use crate::draw::{Corner, Draw, TextFit};
use crate::font::FontFace;
use crate::icon::{IconShape, IconSize};
use crate::text::UiText;
use crate::tokens::rule::UnderEdge;
use crate::tokens::{ColorToken, color};

/// One key of a key chord (Paper: Chip sheet, "Key chord"; Tooltip sheet, shortcut), or a HUD key (Chip sheet,
/// rows "HUD key" and "HUD hint").
pub struct KeyCap {
    pub label: KeyCapLabel,
    pub size: KeyCapSize,
    pub conflict: bool,
}

impl KeyCap {
    /// Paper: Chip sheet, the cap's hard under-edge, `0 2px 0 --color-shadow`.
    const UNDER_EDGE: i8 = 2;

    pub fn desired_width(&self, ctx: &Context) -> f32 {
        let label = match &self.label {
            KeyCapLabel::Text(text) => Draw::measure(ctx, &UiText::resolve(ctx, text), self.size.text()).x,
            KeyCapLabel::Icon(_) => self.size.text().size.points(),
        };
        (label + self.size.padding() * 2.0).max(self.size.min_width()).ceil()
    }

    pub fn paint(&self, painter: &Painter, rect: Rect) {
        let (fill, border, text) = self.colors();
        if self.size == KeyCapSize::Regular {
            let edge = UnderEdge::shadow(Self::UNDER_EDGE);
            painter.add(edge.as_shape(rect, Corner::Small.radius(rect)));
        }
        Draw::fill(painter, rect, Corner::Small, fill);
        if let Some(border) = border {
            Draw::stroke(painter, rect, Corner::Small, border);
        }
        match &self.label {
            KeyCapLabel::Text(label) => {
                let label = UiText::resolve(painter.ctx(), label);
                let galley = Draw::galley(painter, &label, self.size.text(), text, TextFit::Line);
                painter.galley(rect.center() - galley.size() / 2.0, galley, text);
            }
            KeyCapLabel::Icon(icon) => match icon.shape() {
                IconShape::Font(code) => {
                    let font = FontFace::Icons.id(self.size.text().size.points());
                    painter.text(rect.center(), Align2::CENTER_CENTER, code, font, text);
                }
                IconShape::Drawn(_) => icon.paint(painter, rect, IconSize::Small, text),
            },
        }
    }

    pub fn size(&self, ctx: &Context) -> egui::Vec2 {
        vec2(self.desired_width(ctx), self.size.height())
    }

    // (fill, border, text)
    fn colors(&self) -> (Color32, Option<Color32>, Color32) {
        let field = <color::BgField as ColorToken>::VALUE;
        let strong = <color::LineStrong as ColorToken>::VALUE;
        match (self.size, self.conflict) {
            (KeyCapSize::Small, _) => (field, None, <color::TextDim as ColorToken>::VALUE),
            (KeyCapSize::Hud, _) => (field, Some(strong), <color::Text as ColorToken>::VALUE),
            (KeyCapSize::HudHint, _) => (field, Some(strong), <color::TextDim as ColorToken>::VALUE),
            (KeyCapSize::Regular, false) => (field, Some(strong), <color::Text as ColorToken>::VALUE),
            (KeyCapSize::Regular, true) => (
                <color::DangerSoft as ColorToken>::VALUE,
                Some(<color::Danger as ColorToken>::VALUE),
                <color::Danger as ColorToken>::VALUE,
            ),
        }
    }
}
