use amml_i18n::text::Text;
use amml_ui::tokens::{ColorToken, color};
use egui::{Painter, Rect, pos2};

use crate::glyph::EditLockPill;

/// Paper: Menus + Dialogs, `Locked` (`4MU-0`). An inspector body that cannot be edited: a `--color-bg-app` veil at
/// 35 % over it, and the lock pill with the reason along its bottom.
pub struct InspectorLocked {
    reason: Text,
}

impl InspectorLocked {
    // Paper: the veil is at 35 %; the pill is 28 high, 12 in from the sides and the bottom
    const VEIL_ALPHA: u8 = 0x59;
    const PILL_HEIGHT: f32 = 28.0;
    const INSET: f32 = 12.0;

    pub fn new(reason: Text) -> Self {
        Self { reason }
    }

    /// Paints the veil over `body` and the pill along its bottom.
    pub fn paint(&self, painter: &Painter, body: Rect) {
        let veil = <color::BgApp as ColorToken>::VALUE.gamma_multiply_u8(Self::VEIL_ALPHA);
        painter.rect_filled(body, 0, veil);
        let bottom = body.bottom() - Self::INSET;
        let room = Rect::from_min_max(
            pos2(body.left() + Self::INSET, bottom - Self::PILL_HEIGHT),
            pos2(body.right() - Self::INSET, bottom),
        );
        EditLockPill { reason: self.reason.clone() }.paint(painter, room);
    }
}
