use amml_i18n::text::Text;

use super::HeroProblem;

/// What the Library hero card shows.
pub struct HeroCardView {
    pub title: Text,
    /// The facts under the title (length, players, scenes, date), drawn with a dot between them.
    pub meta: Vec<Text>,
    /// The button's words.
    pub open: Text,
    /// Set when the replay cannot be opened: the picture is faint, the button is muted and does nothing, and the
    /// problem is written over the picture.
    pub problem: Option<HeroProblem>,
}
