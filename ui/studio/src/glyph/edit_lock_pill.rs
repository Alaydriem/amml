use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Painter, Rect, Stroke, StrokeKind, pos2, vec2};

use crate::tokens::color as studio;

/// Paper: Assets, `Glyph / edit lock pill` (`3CM-0`). Why a lane or panel cannot be edited: a lock and the reason,
/// in the scene camera colour, on a dark pill.
pub struct EditLockPill {
    pub reason: Text,
}

impl EditLockPill {
    // Paper: padding 4 top and bottom, 10 left and right; `--color-bg-app` at 90 %; a 1 px border at 60 %
    const PADDING_X: f32 = 10.0;
    const PADDING_Y: f32 = 4.0;
    const FILL_ALPHA: u8 = 0xE6;
    const BORDER_ALPHA: u8 = 0x99;

    /// Paints the pill centred in `rect`, no wider than `rect`; a long reason is cut. Returns the pill's rect.
    pub fn paint(&self, painter: &Painter, rect: Rect) -> Rect {
        let hue = <studio::SceneCamera as ColorToken>::VALUE;
        let gap = <spacing::S as SpacingToken>::VALUE;
        let icon = IconSize::Small.points();
        let style = TextStyle::new(TypeSize::Note, FontFace::Semibold);
        let room = (rect.width() - Self::PADDING_X * 2.0 - icon - gap).max(0.0);
        let galley =
            Draw::galley(painter, &UiText::resolve(painter.ctx(), &self.reason), style, hue, TextFit::Truncate(room));
        let content = vec2(icon + gap + galley.size().x, galley.size().y.max(icon));
        let size = content + vec2(Self::PADDING_X, Self::PADDING_Y) * 2.0;
        let pill = Rect::from_center_size(rect.center(), vec2(size.x.min(rect.width()), size.y));
        let fill = <color::BgApp as ColorToken>::VALUE.gamma_multiply_u8(Self::FILL_ALPHA);
        Draw::fill(painter, pill, Corner::Control, fill);
        let border = Stroke::new(1.0, hue.gamma_multiply_u8(Self::BORDER_ALPHA));
        painter.rect_stroke(pill, Corner::Control.radius(pill), border, StrokeKind::Inside);
        let left = pill.left() + Self::PADDING_X;
        let icon_rect = Rect::from_center_size(pos2(left + icon / 2.0, pill.center().y), vec2(icon, icon));
        Icon::Lock.paint(painter, icon_rect, IconSize::Small, hue);
        let text = Rect::from_min_size(pos2(left + icon + gap, pill.center().y - galley.size().y / 2.0), galley.size());
        painter.galley(text.min, galley, hue);
        pill
    }
}
