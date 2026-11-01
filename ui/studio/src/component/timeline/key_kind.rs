use amml_ui::glyph::{Glyph, KeyAuto, KeyCustom, KeyEased, KeyHold, KeyLook};
use egui::{Color32, Painter, Pos2, Rect, Vec2};

/// Paper: `Lanes` (`6WK-0`), the key glyph legend. A key's shape is the easing to the next key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyKind {
    Hold,
    Auto,
    Eased,
    Custom,
}

impl KeyKind {
    // room for the largest key look, its halo included
    const ROOM: f32 = 24.0;

    /// Paints the key centred on `centre`, with the amml-ui key glyph of its kind.
    pub fn paint(self, painter: &Painter, centre: Pos2, color: Color32, selected: bool) {
        let look = if selected { KeyLook::Selected } else { KeyLook::Rest };
        let rect = Rect::from_center_size(centre, Vec2::splat(Self::ROOM));
        match self {
            Self::Hold => KeyHold { look }.paint(painter, rect, color),
            Self::Auto => KeyAuto { look }.paint(painter, rect, color),
            Self::Eased => KeyEased { look }.paint(painter, rect, color),
            Self::Custom => KeyCustom { look }.paint(painter, rect, color),
        }
    }
}
