use egui::Color32;

use crate::icon::Icon;

/// What a menu item shows in its 16 px leading slot. Every kind keeps the slot, so labels line up.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MenuLeading {
    #[default]
    None,
    Icon(Icon),
    /// A check in `--color-accent-hover`: a checked option.
    Check,
    /// A swatch dot in a token colour the caller picks (scene colours).
    Swatch(Color32),
    /// A radio mark, one option out of several: a ring, with an accent dot when it is the chosen option.
    Radio(bool),
}
