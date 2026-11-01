use amml_i18n::text::Text;
use egui::{Color32, CursorIcon, Painter, Rect, Response, Sense, Ui, vec2};

use super::IconButtonSize;
use crate::draw::Draw;
use crate::icon::Icon;
use crate::state::{DisabledReason, InteractionState, StateStyle};
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, color};
use crate::widget::Tooltip;

/// Paper: Widgets, `Widget / Icon button`. A square button with one icon, plain or toggled on.
pub struct IconButton {
    icon: Icon,
    size: IconButtonSize,
    on: bool,
    tooltip: Option<Text>,
    disabled: Option<DisabledReason>,
}

impl IconButton {
    pub fn new(icon: Icon) -> Self {
        Self { icon, size: IconButtonSize::Regular, on: false, tooltip: None, disabled: None }
    }

    pub fn size(mut self, size: IconButtonSize) -> Self {
        self.size = size;
        self
    }

    /// The toggled-on look (`--color-accent-soft` with an `--color-accent-hover` icon).
    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    pub fn tooltip(mut self, text: Text) -> Self {
        self.tooltip = Some(text);
        self
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let side = self.size.side();
        let enabled = self.disabled.is_none();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(vec2(side, side), sense);
        self.paint(ui.painter(), rect, InteractionState::of(&response, enabled));
        match (self.disabled, self.tooltip) {
            (Some(reason), _) => reason.tooltip().attach(response),
            (None, Some(text)) => Tooltip::new(text).attach(response).on_hover_cursor(CursorIcon::PointingHand),
            (None, None) => response.on_hover_cursor(CursorIcon::PointingHand),
        }
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        let painter = state.painter(painter);
        let corner = self.size.corner();
        if state == InteractionState::Focused {
            FocusRing::paint(&painter, rect, corner.radius(rect));
        }
        let style = if self.on { StateStyle::Soft } else { StateStyle::Clear };
        Draw::fill(&painter, rect, corner, style.fill(state));
        self.icon.paint(&painter, rect, self.size.icon(), self.icon_color(state));
    }

    fn icon_color(&self, state: InteractionState) -> Color32 {
        if self.on {
            return <color::AccentHover as ColorToken>::VALUE;
        }
        match state {
            InteractionState::Hover | InteractionState::Pressed | InteractionState::Focused => {
                <color::Text as ColorToken>::VALUE
            }
            InteractionState::Rest | InteractionState::Disabled => <color::TextDim as ColorToken>::VALUE,
        }
    }
}
