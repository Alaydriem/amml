use amml_i18n::text::Text;

use crate::icon::Icon;

/// What a `KeyCap` shows: the key's name, or an icon (an arrow key).
#[derive(Debug, Clone)]
pub enum KeyCapLabel {
    Text(Text),
    Icon(Icon),
}
