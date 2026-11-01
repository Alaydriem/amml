use amml_ui::tokens::{ColorToken, color};
use egui::{Painter, Rect, Shape, Stroke, pos2};

/// Paper: Editor components, `Viewer, edit free camera` (`2LV-0`), the grid overlay. The rule-of-thirds lines and a
/// dashed safe frame 10 % in from each edge, shown while the transport's composition guides toggle is on.
pub struct CompositionGuides;

impl CompositionGuides {
    // Paper: the thirds lines are white at 14 %, the safe frame white at 10 % with 4 px dashes
    const LINE_ALPHA: u8 = 0x24;
    const FRAME_ALPHA: u8 = 0x1A;
    const SAFE_INSET: f32 = 0.1;
    const DASH: f32 = 4.0;

    pub fn paint(painter: &Painter, viewer: Rect) {
        let white = <color::OnAccent as ColorToken>::VALUE;
        let line = Stroke::new(1.0, white.gamma_multiply_u8(Self::LINE_ALPHA));
        for third in [1.0 / 3.0, 2.0 / 3.0] {
            let x = (viewer.left() + viewer.width() * third).floor() + 0.5;
            painter.line_segment([pos2(x, viewer.top()), pos2(x, viewer.bottom())], line);
            let y = (viewer.top() + viewer.height() * third).floor() + 0.5;
            painter.line_segment([pos2(viewer.left(), y), pos2(viewer.right(), y)], line);
        }
        let safe = viewer.shrink2(viewer.size() * Self::SAFE_INSET).shrink(0.5);
        let frame = [safe.left_top(), safe.right_top(), safe.right_bottom(), safe.left_bottom(), safe.left_top()];
        let stroke = Stroke::new(1.0, white.gamma_multiply_u8(Self::FRAME_ALPHA));
        painter.extend(Shape::dashed_line(&frame, stroke, Self::DASH, Self::DASH));
    }
}
