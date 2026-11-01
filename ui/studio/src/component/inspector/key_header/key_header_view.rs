use amml_i18n::text::Text;

use super::KeyHeaderTone;

/// What the top of the Key tab shows: a key, several keys or a track.
pub struct KeyHeaderView {
    pub title: Text,
    /// The line under the title: the key's time, the tracks of several keys, or the scene of a track.
    pub detail: Text,
    /// The small pill at the right: the key's place, "multi", or the key count.
    pub chip: Text,
    pub tone: KeyHeaderTone,
}
