use amml_i18n::text::Text;

use super::SpanLook;

/// A stretch of a lane from `start` to `end` seconds, with an optional label and its look. An `Ease` span needs its
/// label (the easing name); without one it is not drawn.
#[derive(Debug, Clone)]
pub struct LaneSpan {
    pub start: f64,
    pub end: f64,
    pub label: Option<Text>,
    pub look: SpanLook,
}
