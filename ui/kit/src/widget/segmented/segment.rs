use amml_i18n::text::Text;

use crate::icon::Icon;

/// One segment of a `Segmented` control: a word or an icon.
pub enum Segment {
    Text(Text),
    Icon(Icon),
}
