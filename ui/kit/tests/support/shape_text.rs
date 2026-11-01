use egui::Shape;

// the text of every text shape, nested shapes included
pub struct ShapeText;

impl ShapeText {
    pub fn all(shapes: &[Shape]) -> Vec<String> {
        let mut out = Vec::new();
        for shape in shapes {
            match shape {
                Shape::Text(text) => out.push(text.galley.text().to_owned()),
                Shape::Vec(inner) => out.extend(Self::all(inner)),
                _ => {}
            }
        }
        out
    }

    pub fn contains(shapes: &[Shape], needle: &str) -> bool {
        Self::all(shapes).iter().any(|text| text.contains(needle))
    }
}
