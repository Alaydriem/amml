use amml_i18n::text::Text;
use egui::Color32;

/// One row of a `LegendCard`: a square swatch in the colour the picture uses, and what it means.
#[derive(Debug, Clone)]
pub struct LegendRow {
    colour: Color32,
    label: Text,
}

impl LegendRow {
    pub fn new(colour: Color32, label: Text) -> Self {
        Self { colour, label }
    }

    pub fn colour(&self) -> Color32 {
        self.colour
    }

    pub fn label(&self) -> &Text {
        &self.label
    }
}
