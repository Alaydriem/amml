use std::ops::RangeInclusive;

use egui::{Color32, CursorIcon, Key, Painter, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Corner, Draw};
use crate::state::{ArrowKeys, DisabledReason, InteractionState};
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, color};

/// Paper: Widgets, `Widget / Slider`. A track with an accent fill and a knob with an accent dot;
/// the knob grows on hover, a halo shows while it is dragged, and steps can show as ticks.
pub struct Slider<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    step: f64,
    ticks: bool,
    disabled: Option<DisabledReason>,
}

impl<'a> Slider<'a> {
    /// Paper: Slider sheet, the control height (24 with ticks).
    pub const HEIGHT: f32 = 20.0;
    pub const HEIGHT_WITH_TICKS: f32 = 24.0;
    /// Paper: Slider sheet, the track height.
    const TRACK: f32 = 4.0;
    /// Paper: Slider sheet, the knob side at rest, and on hover or while dragged.
    const KNOB: f32 = 14.0;
    const KNOB_HOVER: f32 = 16.0;
    /// Paper: Slider sheet, the dot in the knob and the halo while dragged.
    const DOT: f32 = 5.0;
    const HALO: f32 = 28.0;
    /// Paper: Slider sheet, the tick side, and its centre below the track centre.
    const TICK: f32 = 3.0;
    const TICK_DROP: f32 = 9.5;
    /// The most ticks a stepped slider shows.
    const MAX_TICKS: f64 = 20.0;
    /// The share of the range an arrow key moves a continuous slider.
    const KEY_FRACTION: f64 = 0.01;

    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self { value, range, step: 0.0, ticks: false, disabled: None }
    }

    /// Snaps the value to `start + n * step`. Zero or less is continuous.
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Shows a tick at each step when there are 1 to 20 steps.
    pub fn ticks(mut self, ticks: bool) -> Self {
        self.ticks = ticks;
        self
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    pub fn show(self, ui: &mut Ui, width: f32) -> Response {
        let enabled = self.disabled.is_none();
        let sense = if enabled { Sense::click_and_drag() } else { Sense::hover() };
        let height = if self.tick_count().is_some() { Self::HEIGHT_WITH_TICKS } else { Self::HEIGHT };
        let (rect, mut response) = ui.allocate_exact_size(vec2(width, height), sense);
        let before = self.current();
        if enabled {
            if let Some(pointer) = response.interact_pointer_pos().filter(|_| response.is_pointer_button_down_on()) {
                let track = Self::track_x(rect);
                let fraction =
                    if track.span() > 0.0 { ((pointer.x - track.min) / track.span()).clamp(0.0, 1.0) } else { 0.0 };
                *self.value = self.snap(self.start() + f64::from(fraction) * self.span());
            }
            if response.has_focus() {
                ArrowKeys::keep(ui, response.id);
                let delta = ui.input(|input| {
                    i32::from(input.key_pressed(Key::ArrowRight) || input.key_pressed(Key::ArrowUp))
                        - i32::from(input.key_pressed(Key::ArrowLeft) || input.key_pressed(Key::ArrowDown))
                });
                if delta != 0 {
                    let unit = if self.step > 0.0 { self.step } else { self.span() * Self::KEY_FRACTION };
                    *self.value = self.snap(self.current() + f64::from(delta) * unit);
                }
            }
        }
        if (self.current() - before).abs() > f64::EPSILON {
            response.mark_changed();
        }
        let state = InteractionState::of(&response, enabled);
        self.paint(ui.painter(), rect, state);
        match &self.disabled {
            None => response.on_hover_cursor(CursorIcon::PointingHand),
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    /// Paints the slider; Pressed is the dragging look (halo).
    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        let painter = state.painter(painter);
        let active = matches!(state, InteractionState::Hover | InteractionState::Pressed);
        let track_x = Self::track_x(rect);
        let centre_y = rect.top() + Self::HEIGHT / 2.0;
        let knob_x = track_x.min + track_x.span() * self.fraction();
        let track = Rect::from_min_max(
            pos2(track_x.min, centre_y - Self::TRACK / 2.0),
            pos2(track_x.max, centre_y + Self::TRACK / 2.0),
        );
        let (track_fill, fill) = if active {
            (<color::BgActive as ColorToken>::VALUE, <color::AccentHover as ColorToken>::VALUE)
        } else {
            (<color::BgHover as ColorToken>::VALUE, <color::Accent as ColorToken>::VALUE)
        };
        Draw::fill(&painter, track, Corner::Full, track_fill);
        Draw::fill(&painter, Rect::from_min_max(track.min, pos2(knob_x, track.max.y)), Corner::Full, fill);
        self.paint_ticks(&painter, track_x, centre_y, knob_x);
        let knob = pos2(knob_x, centre_y);
        if state == InteractionState::Pressed {
            painter.circle_filled(knob, Self::HALO / 2.0, <color::AccentSoft as ColorToken>::VALUE);
        }
        let side = if active { Self::KNOB_HOVER } else { Self::KNOB };
        if state == InteractionState::Focused {
            let knob_rect = Rect::from_center_size(knob, vec2(side, side));
            FocusRing::paint(&painter, knob_rect, Corner::Full.radius(knob_rect));
        }
        painter.circle_filled(knob, side / 2.0, <color::OnAccent as ColorToken>::VALUE);
        painter.circle_filled(knob, Self::DOT / 2.0, <color::Accent as ColorToken>::VALUE);
    }

    fn paint_ticks(&self, painter: &Painter, track_x: egui::Rangef, centre_y: f32, knob_x: f32) {
        let Some(count) = self.tick_count() else { return };
        for index in 0..=count {
            let x = track_x.min + track_x.span() * index as f32 / count as f32;
            let lit = x <= knob_x + 0.5;
            let fill: Color32 =
                if lit { <color::Accent as ColorToken>::VALUE } else { <color::LineStrong as ColorToken>::VALUE };
            painter.circle_filled(pos2(x, centre_y + Self::TICK_DROP), Self::TICK / 2.0, fill);
        }
    }

    // the number of steps when ticks show: 1 to 20
    fn tick_count(&self) -> Option<usize> {
        if !self.ticks || self.step <= 0.0 || self.span() <= 0.0 {
            return None;
        }
        let steps = (self.span() / self.step).round();
        // Checked to 1..=20 first, so the cast cannot truncate.
        (1.0..=Self::MAX_TICKS).contains(&steps).then_some(steps as usize)
    }

    // the knob runs between the knob radius in from each side
    fn track_x(rect: Rect) -> egui::Rangef {
        let inset = Self::KNOB / 2.0;
        egui::Rangef::new(rect.left() + inset, (rect.right() - inset).max(rect.left() + inset))
    }

    fn fraction(&self) -> f32 {
        if self.span() <= 0.0 {
            return 0.0;
        }
        ((self.current() - self.start()) / self.span()).clamp(0.0, 1.0) as f32
    }

    // the value, with NaN read as the start of the range
    fn current(&self) -> f64 {
        if self.value.is_nan() { self.start() } else { *self.value }
    }

    fn snap(&self, raw: f64) -> f64 {
        let snapped =
            if self.step > 0.0 { self.start() + ((raw - self.start()) / self.step).round() * self.step } else { raw };
        snapped.clamp(self.start(), self.end().max(self.start()))
    }

    fn start(&self) -> f64 {
        *self.range.start()
    }

    fn end(&self) -> f64 {
        *self.range.end()
    }

    fn span(&self) -> f64 {
        (self.end() - self.start()).max(0.0)
    }
}
