use egui::epaint::Shadow;

use crate::tokens::{ColorToken, color};

/// A hard edge under a raised part, as Paper writes it: `0 <depth>px 0 --color-shadow` (Chip sheet, key cap). Paper has
/// no shadow token type, so the rule is written here once.
pub struct UnderEdge;

impl UnderEdge {
    pub fn shadow(depth: i8) -> Shadow {
        Shadow { offset: [0, depth], blur: 0, spread: 0, color: <color::Shadow as ColorToken>::VALUE }
    }
}
