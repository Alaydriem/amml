use amml_ui::font::{FontLoader, FontSource};
use egui::{Context, Painter, Pos2, RawInput, Rect, Shape, vec2};

// runs headless frames with the built-in fonts installed and returns what one paint call drew
pub struct PaintCapture;

impl PaintCapture {
    pub fn shapes(mut paint: impl FnMut(&Painter)) -> Vec<Shape> {
        let ctx = Context::default();
        FontLoader::install(&ctx, FontSource::BuiltIn);
        // the new fonts take effect on the next pass
        ctx.run_ui(Self::input(), |_| {}).textures_delta.clear();
        let mut output = ctx.run_ui(Self::input(), |ui| paint(ui.painter()));
        output.textures_delta.clear();
        output.shapes.into_iter().map(|clipped| clipped.shape).collect()
    }

    // the union of every painted shape's visual bounds
    pub fn bounds(shapes: &[Shape]) -> Rect {
        shapes.iter().fold(Rect::NOTHING, |all, shape| all.union(shape.visual_bounding_rect()))
    }

    // the painted rect of each text shape
    pub fn text_rects(shapes: &[Shape]) -> Vec<Rect> {
        shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Text(text) => Some(text.galley.rect.translate(text.pos.to_vec2())),
                _ => None,
            })
            .collect()
    }

    fn input() -> RawInput {
        RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))), ..Default::default() }
    }
}
