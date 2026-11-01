use amml_i18n::text::Text;

use super::EasingFamily;

/// A named curve: its name, its y at even steps of x from 0 to 1 (as `CurvePlotView::samples`), and its family.
#[derive(Debug, Clone)]
pub struct EasingPreset {
    pub name: Text,
    pub samples: Vec<f32>,
    pub family: EasingFamily,
}
