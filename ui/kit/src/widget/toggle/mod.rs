use egui::{Color32, CursorIcon, Painter, Rect, Response, Sense, Ui, Vec2, pos2, vec2};

use crate::draw::{Corner, Draw};
use crate::motion::Motion;
use crate::state::{DisabledReason, InteractionState, StateStyle};
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, color};

/// Paper: Widgets, `Widget / Toggle`. An on/off switch; the knob slides when it changes.
pub struct Toggle<'a> {
    on: &'a mut bool,
    disabled: Option<DisabledReason>,
}

impl<'a> Toggle<'a> {
    /// Paper: Toggle sheet, the track.
    pub const TRACK: Vec2 = vec2(34.0, 20.0);
    /// Paper: Toggle sheet, the knob side.
    pub const KNOB: f32 = 14.0;
    /// Paper: Toggle sheet, the gap between the knob and the track edge.
    pub const INSET: f32 = 3.0;

    pub fn new(on: &'a mut bool) -> Self {
        Self { on, disabled: None }
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let enabled = self.disabled.is_none();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, mut response) = ui.allocate_exact_size(Self::TRACK, sense);
        if response.clicked() {
            *self.on = !*self.on;
            response.mark_changed();
        }
        let state = InteractionState::of(&response, enabled);
        let amount = Motion::flag(ui.ctx(), response.id, *self.on);
        Self::paint_amount(ui.painter(), rect, state, amount);
        match &self.disabled {
            None => response.on_hover_cursor(CursorIcon::PointingHand),
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        Self::paint_amount(painter, rect, state, if *self.on { 1.0 } else { 0.0 });
    }

    /// Paints the switch `amount` of the way from off (0) to on (1), so the knob can slide.
    pub fn paint_amount(painter: &Painter, rect: Rect, state: InteractionState, amount: f32) {
        let painter = state.painter(painter);
        if state == InteractionState::Focused {
            FocusRing::paint(&painter, rect, Corner::Full.radius(rect));
        }
        let track = Self::off_track(state).lerp_to_gamma(StateStyle::Accent.fill(state), amount);
        let knob = Self::off_knob(state).lerp_to_gamma(<color::OnAccent as ColorToken>::VALUE, amount);
        Draw::fill(&painter, rect, Corner::Full, track);
        let radius = Self::KNOB / 2.0;
        let start = rect.left() + Self::INSET + radius;
        let end = rect.right() - Self::INSET - radius;
        painter.circle_filled(pos2(start + (end - start) * amount, rect.center().y), radius, knob);
    }

    // Paper's off row: the track starts one step up (bg-hover) and presses to line-strong
    fn off_track(state: InteractionState) -> Color32 {
        match state {
            InteractionState::Hover => <color::BgActive as ColorToken>::VALUE,
            InteractionState::Pressed => <color::LineStrong as ColorToken>::VALUE,
            InteractionState::Rest | InteractionState::Focused | InteractionState::Disabled => {
                <color::BgHover as ColorToken>::VALUE
            }
        }
    }

    fn off_knob(state: InteractionState) -> Color32 {
        match state {
            InteractionState::Hover | InteractionState::Pressed => <color::Text as ColorToken>::VALUE,
            InteractionState::Rest | InteractionState::Focused | InteractionState::Disabled => {
                <color::TextDim as ColorToken>::VALUE
            }
        }
    }
}
