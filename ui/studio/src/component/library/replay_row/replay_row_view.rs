use amml_i18n::text::Text;

use super::ReplayRowNote;
use crate::component::library::HeroPicture;

/// What one row of the Library's replay list shows. Every word comes from the host.
pub struct ReplayRowView {
    pub name: Text,
    /// The replay's length, written on the strip at the foot of the thumbnail.
    pub length: Text,
    /// The facts under the name (date, players, scenes, bookmarks), drawn with a dot between them.
    pub facts: Vec<Text>,
    pub note: ReplayRowNote,
    /// The picture of the thumbnail; without one the thumbnail is a plain card.
    pub thumbnail: Option<HeroPicture>,
    pub selected: bool,
}
