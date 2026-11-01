use amml_i18n::text::Text;

/// Why a replay cannot be opened: the words a broken hero card draws over its faded picture.
pub struct HeroProblem {
    pub title: Text,
    /// One line that says what is wrong.
    pub reason: Text,
}
