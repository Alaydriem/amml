use amml_ui::font::{FontLoader, FontSource};
use egui::epaint::RectShape;
use egui::{Context, FontFamily, Painter, Pos2, RawInput, Rect, Shape, vec2};

// runs headless frames with the built-in fonts installed and returns what one paint call drew
pub struct PaintCapture;

impl PaintCapture {
    pub fn shapes(paint: impl FnMut(&Painter)) -> Vec<Shape> {
        Self::shapes_in(&Context::default(), 0.0, paint)
    }

    // paints in `ctx` (which may carry a locale) at input time `time`
    pub fn shapes_in(ctx: &Context, time: f64, mut paint: impl FnMut(&Painter)) -> Vec<Shape> {
        FontLoader::install(ctx, FontSource::BuiltIn);
        // the new fonts take effect on the next pass
        ctx.run_ui(Self::input(time), |_| {}).textures_delta.clear();
        let mut output = ctx.run_ui(Self::input(time), |ui| paint(ui.painter()));
        output.textures_delta.clear();
        output.shapes.into_iter().map(|clipped| clipped.shape).collect()
    }

    // each text shape as (text, font size, family)
    pub fn texts(shapes: &[Shape]) -> Vec<(String, f32, FontFamily)> {
        let mut out = Vec::new();
        for shape in shapes {
            if let Shape::Text(text) = shape {
                let font = text.galley.job.sections.first().map(|section| section.format.font_id.clone());
                if let Some(font) = font {
                    out.push((text.galley.text().to_owned(), font.size, font.family));
                }
            }
        }
        out
    }

    // each text shape as (text, painted rect)
    pub fn text_rects(shapes: &[Shape]) -> Vec<(String, Rect)> {
        shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Text(text) => {
                    Some((text.galley.text().to_owned(), text.galley.rect.translate(text.pos.to_vec2())))
                }
                _ => None,
            })
            .collect()
    }

    // every rect shape, nested shapes included
    pub fn rects(shapes: &[Shape]) -> Vec<RectShape> {
        let mut out = Vec::new();
        for shape in shapes {
            match shape {
                Shape::Rect(rect) => out.push(rect.clone()),
                Shape::Vec(inner) => out.extend(Self::rects(inner)),
                _ => {}
            }
        }
        out
    }

    fn input(time: f64) -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            time: Some(time),
            ..Default::default()
        }
    }
}
