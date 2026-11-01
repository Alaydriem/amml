use amml_ui::draw::{Corner, Draw};
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::epaint::RectShape;
use egui::{CornerRadius, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2, pos2};

use super::CurvePlotView;

/// Paper: Widgets, `Widget / Curve plot` (`6IV-0`), the plot in each curve card. An easing curve in a framed unit box
/// on `--color-bg-app`, with a 4 × 4 grid, a key diamond at each end and, for a custom curve, its two handles.
pub struct CurvePlot<'a> {
    view: &'a CurvePlotView<'a>,
}

impl<'a> CurvePlot<'a> {
    /// Paper: the curve is `--spacing-stroke-curve` wide.
    pub const CURVE_WIDTH: f32 = <spacing::StrokeCurve as SpacingToken>::VALUE;
    // Paper: the box is 8 in from the plot's edge, split in four each way
    const INSET: f32 = 8.0;
    const GRID: usize = 4;
    // Paper: the handle knobs are radius 4 with a `--spacing-stroke-thin` ring; the end keys are diamonds of half 6
    const KNOB_RADIUS: f32 = 4.0;
    const END_HALF: f32 = 6.0;

    pub fn new(view: &'a CurvePlotView<'a>) -> Self {
        Self { view }
    }

    /// The unit box inside a plot drawn in `rect`: x = 0 at its left, y = 0 at its bottom.
    pub fn plot_box(rect: Rect) -> Rect {
        rect.shrink(Self::INSET)
    }

    /// Where the point (`x`, `y`) of the unit box is drawn on a plot in `rect`. An overshooting curve or handle
    /// stays on the plot, so a knob is always where it can be grabbed.
    pub fn point(rect: Rect, x: f32, y: f32) -> Pos2 {
        let plot = Self::plot_box(rect);
        let room = rect.shrink(Self::KNOB_RADIUS + <spacing::StrokeThin as SpacingToken>::VALUE);
        pos2(plot.left() + plot.width() * x, plot.bottom() - plot.height() * y).clamp(room.min, room.max)
    }

    pub fn paint(&self, painter: &Painter, rect: Rect) {
        let line = <color::Line as ColorToken>::VALUE;
        Draw::fill(painter, rect, Corner::Control, <color::BgApp as ColorToken>::VALUE);
        let plot = Self::plot_box(rect);
        painter.rect_stroke(plot, 0, Stroke::new(1.0, line), StrokeKind::Middle);
        for step in 1..Self::GRID {
            let part = step as f32 / Self::GRID as f32;
            let x = plot.left() + plot.width() * part;
            let y = plot.top() + plot.height() * part;
            painter.line_segment([pos2(x, plot.top()), pos2(x, plot.bottom())], Stroke::new(1.0, line));
            painter.line_segment([pos2(plot.left(), y), pos2(plot.right(), y)], Stroke::new(1.0, line));
        }
        let at = |x: f32, y: f32| Self::point(rect, x, y);
        if let Some([first, second]) = self.view.handles {
            let guide = Stroke::new(1.0, <color::TextFaint as ColorToken>::VALUE);
            painter.line_segment([at(0.0, 0.0), at(first.x, first.y)], guide);
            painter.line_segment([at(1.0, 1.0), at(second.x, second.y)], guide);
        }
        self.curve(painter, &at);
        if let Some(handles) = self.view.handles {
            let ring = Stroke::new(<spacing::StrokeThin as SpacingToken>::VALUE, self.view.color);
            for handle in handles {
                painter.circle(at(handle.x, handle.y), Self::KNOB_RADIUS, <color::BgApp as ColorToken>::VALUE, ring);
            }
        }
        for end in [plot.left_bottom(), plot.right_top()] {
            Self::diamond(painter, end, self.view.color);
        }
    }

    // the samples at even steps of x, as one line
    fn curve(&self, painter: &Painter, at: &impl Fn(f32, f32) -> Pos2) {
        let samples = self.view.samples;
        if samples.len() < 2 {
            return;
        }
        let last = (samples.len() - 1) as f32;
        let points = samples.iter().enumerate().map(|(index, y)| at(index as f32 / last, *y)).collect();
        painter.add(Shape::line(points, Stroke::new(Self::CURVE_WIDTH, self.view.color)));
    }

    fn diamond(painter: &Painter, centre: Pos2, color: egui::Color32) {
        let rect = Rect::from_center_size(centre, Vec2::splat(Self::END_HALF * std::f32::consts::SQRT_2));
        let shape = RectShape::filled(rect, CornerRadius::ZERO, color)
            .with_round_to_pixels(false)
            .with_angle(std::f32::consts::FRAC_PI_4);
        painter.add(shape);
    }
}
