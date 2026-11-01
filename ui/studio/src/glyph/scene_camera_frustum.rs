use amml_ui::glyph::Glyph;
use amml_ui::tokens::rule::SoftFill;
use egui::{Color32, Painter, Rect, Shape, Stroke, pos2};

/// Paper: Assets, `Glyph / scene camera frustum` (`39Y-0`). The scene camera's view: a cone from its eye at the left
/// of the rect to the right edge, with an arrow on the far edge and a dot at the eye. A filled frustum has a body at
/// `--opacity-soft`. The caller gives the colour (`--color-scene-camera`, or `--color-playhead` for the free camera).
pub struct SceneCameraFrustum {
    pub filled: bool,
}

impl SceneCameraFrustum {
    // Paper (at 1x): a 1.1 px outline over a 2 px black shadow at 27 %
    const OUTLINE: f32 = 1.1;
    const SHADOW: f32 = 2.0;
    const SHADOW_ALPHA: u8 = 0x45;
    // Paper (at 1x): the eye dot is radius 2; the arrow is 11 across and 6 deep
    const DOT_RADIUS: f32 = 2.0;
    const ARROW_HALF: f32 = 5.5;
    const ARROW_DEPTH: f32 = 6.0;
}

impl Glyph for SceneCameraFrustum {
    fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        // the outline shadow and the eye dot reach this far past the cone
        let inset = Self::SHADOW.max(Self::DOT_RADIUS);
        let inner = rect.shrink(inset);
        let eye = inner.left_center();
        let far = inner.right() - Self::ARROW_DEPTH;
        let cone = vec![eye, pos2(far, inner.top()), pos2(far, inner.bottom())];
        let shadow = Color32::BLACK.gamma_multiply_u8(Self::SHADOW_ALPHA);
        painter.add(Shape::closed_line(cone.clone(), Stroke::new(Self::SHADOW, shadow)));
        let outline = Stroke::new(Self::OUTLINE, color);
        if self.filled {
            painter.add(Shape::convex_polygon(cone, SoftFill::of(color), outline));
        } else {
            painter.add(Shape::closed_line(cone, outline));
        }
        let y = inner.center().y;
        let arrow =
            vec![pos2(far, y - Self::ARROW_HALF), pos2(far + Self::ARROW_DEPTH, y), pos2(far, y + Self::ARROW_HALF)];
        painter.add(Shape::convex_polygon(arrow, color, Stroke::NONE));
        painter.circle_filled(eye, Self::DOT_RADIUS, color);
    }
}
