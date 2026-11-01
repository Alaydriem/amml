mod lane_key;
mod lane_span;
mod lane_view;
mod span_look;

pub use lane_key::LaneKey;
pub use lane_span::LaneSpan;
pub use lane_view::LaneView;
pub use span_look::SpanLook;

use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Align2, Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

use super::TimeAxis;
use crate::glyph::{CameraSegment, EditLockPill};
use crate::rule::LabelTint;

/// Paper: `Lanes` (`6WK-0`) and `Box select, multi-select, lock` (`7J7-0`). The time area of one track,
/// `--spacing-lane` high: the active scene's range, the spans, the value curve and the keys. A hidden track is
/// veiled; a locked one is dimmed and hatched, with its lock pill.
pub struct Lane<'a> {
    view: &'a LaneView,
    axis: &'a TimeAxis,
    hidden: bool,
    locked: bool,
    scene: Option<(f64, f64, Color32)>,
}

impl<'a> Lane<'a> {
    // Paper: spans are 24 high at 20 from the top
    const SPAN_TOP: f32 = 20.0;
    const SPAN_HEIGHT: f32 = 24.0;
    const SPAN_PADDING: f32 = 14.0;
    // Paper: a fill runs from 5 % to 25 % with its strength, a clip from 12 % to 24 % with a 45 % border
    const FILL_ALPHA: [f32; 2] = [0.05, 0.25];
    const CLIP_ALPHA: [f32; 2] = [0.12, 0.24];
    const CLIP_BORDER_ALPHA: u8 = 0x73;
    // Paper: the curve keeps 4 from the top and bottom, its area at 22 %
    const CURVE_INSET: f32 = 4.0;
    const CURVE_FILL_ALPHA: u8 = 0x38;
    // Paper: the active scene's range is its colour at 6 %
    const SCENE_ALPHA: u8 = 0x0F;
    // Paper: a hidden lane has a `--color-bg-app` veil at 50 % and keys at 35 %
    const VEIL_ALPHA: u8 = 0x80;
    const HIDDEN_KEY_ALPHA: u8 = 0x59;
    // Paper: a locked lane is hatched every 14 in its colour at 8 %
    const HATCH: f32 = 14.0;
    const HATCH_ALPHA: u8 = 0x14;
    const LOCK_RIGHT: f32 = 24.0;
    // a press this close to a key takes the key
    const HIT: f32 = 8.0;

    pub fn new(view: &'a LaneView, axis: &'a TimeAxis) -> Self {
        Self { view, axis, hidden: false, locked: false, scene: None }
    }

    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    /// The active scene's start, end and colour, tinted behind the lane.
    pub fn scene(mut self, scene: Option<(f64, f64, Color32)>) -> Self {
        self.scene = scene;
        self
    }

    /// The index of the key nearest `at` within reach, on a lane at `rect`.
    pub fn key_at(&self, rect: Rect, at: Pos2) -> Option<usize> {
        if (at.y - rect.center().y).abs() > Self::HIT {
            return None;
        }
        self.view
            .keys
            .iter()
            .enumerate()
            .map(|(index, key)| (index, (self.axis.x_of(key.seconds) - at.x).abs()))
            .filter(|(_, distance)| *distance <= Self::HIT)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    pub fn paint(&self, painter: &Painter, rect: Rect) {
        painter.rect_filled(rect, 0, <color::BgPanel as ColorToken>::VALUE);
        Draw::divider(painter, rect.x_range(), rect.bottom() - 1.0, <color::Line as ColorToken>::VALUE);
        let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
        let hue = self.view.color.color();
        if let Some((start, end, scene)) = self.scene {
            let range =
                Rect::from_x_y_ranges(self.axis.x_of(start)..=self.axis.x_of(end), rect.top()..=rect.bottom() - 1.0);
            painter.rect_filled(range, 0, scene.gamma_multiply_u8(Self::SCENE_ALPHA));
        }
        for span in &self.view.spans {
            self.span(&painter, rect, span, hue);
        }
        self.curve(&painter, rect, hue);
        if self.locked {
            painter.rect_filled(rect, 0, <color::Dim as ColorToken>::VALUE);
            Self::hatch(&painter, rect, hue.gamma_multiply_u8(Self::HATCH_ALPHA));
        }
        if self.hidden {
            painter.rect_filled(rect, 0, <color::BgApp as ColorToken>::VALUE.gamma_multiply_u8(Self::VEIL_ALPHA));
        }
        let key_hue = if self.hidden { hue.gamma_multiply_u8(Self::HIDDEN_KEY_ALPHA) } else { hue };
        for key in &self.view.keys {
            key.kind.paint(&painter, pos2(self.axis.x_of(key.seconds), rect.center().y), key_hue, key.selected);
        }
        if self.locked
            && let Some(reason) = &self.view.lock_reason
        {
            let room = Rect::from_min_max(
                pos2(rect.center().x, rect.top()),
                pos2(rect.right() - Self::LOCK_RIGHT, rect.bottom()),
            );
            EditLockPill { reason: reason.clone() }.paint(&painter, room);
        }
    }

    fn span(&self, painter: &Painter, rect: Rect, span: &LaneSpan, hue: Color32) {
        let (left, right) = (self.axis.x_of(span.start), self.axis.x_of(span.end));
        if let SpanLook::Ease { hold } = span.look {
            // the camera segment always names its easing; a span without one is not drawn
            if let Some(ease) = span.label.clone() {
                let half = CameraSegment::HEIGHT / 2.0;
                let row = Rect::from_x_y_ranges(left..=right, rect.center().y - half..=rect.center().y + half);
                CameraSegment { ease, hold }.paint(painter, row);
            }
            return;
        }
        let place = Rect::from_min_size(pos2(left, rect.top() + Self::SPAN_TOP), vec2(right - left, Self::SPAN_HEIGHT));
        let ink = LabelTint::of(hue);
        match span.look {
            SpanLook::Fill { strength } => {
                let alpha = Self::mix(Self::FILL_ALPHA, strength);
                Draw::fill(painter, place, Corner::Small, hue.gamma_multiply(alpha));
                if let Some(label) = &span.label {
                    let style = TextStyle::new(TypeSize::Micro, FontFace::Regular);
                    let galley =
                        Draw::galley(painter, &UiText::resolve(painter.ctx(), label), style, ink, TextFit::Line);
                    if galley.size().x + Self::SPAN_PADDING <= place.width() {
                        painter.galley(Rect::from_center_size(place.center(), galley.size()).min, galley, ink);
                    }
                }
            }
            SpanLook::Clip { strength } => {
                let alpha = Self::mix(Self::CLIP_ALPHA, strength);
                Draw::fill(painter, place, Corner::Small, hue.gamma_multiply(alpha));
                Draw::stroke(painter, place, Corner::Small, hue.gamma_multiply_u8(Self::CLIP_BORDER_ALPHA));
                if let Some(label) = &span.label {
                    let style = TextStyle::new(TypeSize::Caption, FontFace::Semibold);
                    let room = (place.width() - Self::SPAN_PADDING * 2.0).max(0.0);
                    let at = pos2(place.left() + Self::SPAN_PADDING, place.center().y);
                    Draw::text(painter, at, Align2::LEFT_CENTER, label, style, ink, TextFit::Truncate(room));
                }
            }
            SpanLook::Ease { .. } => {}
        }
    }

    // the value curve and the area under it
    fn curve(&self, painter: &Painter, rect: Rect, hue: Color32) {
        if self.view.curve.len() < 2 {
            return;
        }
        let bottom = rect.bottom() - 1.0;
        let height = rect.height() - 1.0 - Self::CURVE_INSET * 2.0;
        let points: Vec<Pos2> = self
            .view
            .curve
            .iter()
            .map(|(seconds, value)| {
                pos2(self.axis.x_of(*seconds), bottom - Self::CURVE_INSET - value.clamp(0.0, 1.0) * height)
            })
            .collect();
        let area = hue.gamma_multiply_u8(Self::CURVE_FILL_ALPHA);
        for pair in points.windows(2) {
            let quad = vec![pair[0], pair[1], pos2(pair[1].x, bottom), pos2(pair[0].x, bottom)];
            painter.add(Shape::convex_polygon(quad, area, Stroke::NONE));
        }
        painter.add(Shape::line(points, Stroke::new(<spacing::StrokeThin as SpacingToken>::VALUE, hue)));
    }

    // diagonal lines across `rect`, top left to bottom right
    fn hatch(painter: &Painter, rect: Rect, ink: Color32) {
        let stroke = Stroke::new(1.0, ink);
        let mut offset = -rect.height();
        while offset < rect.width() {
            let from = pos2(rect.left() + offset, rect.top());
            painter.line_segment([from, from + vec2(rect.height(), rect.height())], stroke);
            offset += Self::HATCH;
        }
    }

    fn mix([low, high]: [f32; 2], strength: f32) -> f32 {
        low + (high - low) * strength.clamp(0.0, 1.0)
    }
}
