use amml_i18n::text::UserText;

use crate::icon::Icon;

/// What a corner button shows at its left: an icon (the Mods button), or a mod's short mark on a tile.
#[derive(Debug, Clone)]
pub enum CornerButtonLead {
    Icon(Icon),
    /// One to three characters the mod chose, such as "DV".
    Mark(UserText),
}
