use amml_i18n::text::Text;

/// A bookmark on the ruler: its time in seconds and its name.
#[derive(Debug, Clone)]
pub struct TimelineBookmark {
    pub seconds: f64,
    pub label: Text,
}
