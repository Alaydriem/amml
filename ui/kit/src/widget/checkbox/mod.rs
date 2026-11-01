use amml_i18n::text::Text;
use egui::{CursorIcon, Painter, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::{Icon, IconSize};
use crate::state::{DisabledReason, InteractionState, Selection, StateStyle};
use crate::text::UiText;
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Checkbox`. A box that is off, on or mixed; with a label it is a list
/// row whose whole area is the click target.
pub struct Checkbox<'a> {
    value: &'a mut Selection,
    label: Option<Text>,
    indent: bool,
    disabled: Option<DisabledReason>,
}

impl<'a> Checkbox<'a> {
    /// Paper: Checkbox sheet, the height of the mixed dash.
    const DASH_HEIGHT: f32 = 2.0;

    pub fn new(value: &'a mut Selection) -> Self {
        Self { value, label: None, indent: false, disabled: None }
    }

    /// A list row with this label next to the box.
    pub fn label(mut self, label: Text) -> Self {
        self.label = Some(label);
        self
    }

    /// Indents the row as a child of a parent checkbox (`--spacing-row` from the row edge).
    pub fn indent(mut self, indent: bool) -> Self {
        self.indent = indent;
        self
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let enabled = self.disabled.is_none();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let size = match self.label {
            Some(_) => vec2(ui.available_width(), <spacing::Field as SpacingToken>::VALUE),
            None => vec2(Self::side(), Self::side()),
        };
        let (rect, mut response) = ui.allocate_exact_size(size, sense);
        if response.clicked() {
            *self.value = match *self.value {
                Selection::On => Selection::Off,
                Selection::Off | Selection::Mixed => Selection::On,
            };
            response.mark_changed();
        }
        let state = InteractionState::of(&response, enabled);
        match self.label {
            Some(_) => self.paint_row(ui.painter(), rect, state),
            None => self.paint(ui.painter(), rect, state),
        }
        match &self.disabled {
            None => response.on_hover_cursor(CursorIcon::PointingHand),
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    /// Paints the box alone into `rect`.
    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        let painter = state.painter(painter);
        if state == InteractionState::Focused {
            FocusRing::paint(&painter, rect, Corner::Small.radius(rect));
        }
        self.paint_box(&painter, rect, state);
    }

    /// Paints the list row: the hover fill, the box and the label. The row takes the ring.
    pub fn paint_row(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        let painter = state.painter(painter);
        if state == InteractionState::Focused {
            FocusRing::paint(&painter, rect, Corner::Control.radius(rect));
        }
        Draw::fill(&painter, rect, Corner::Control, StateStyle::Clear.fill(state));
        let gap = <spacing::S as SpacingToken>::VALUE;
        let left = rect.left() + if self.indent { <spacing::Row as SpacingToken>::VALUE } else { gap };
        let box_rect =
            Rect::from_min_size(pos2(left, rect.center().y - Self::side() / 2.0), vec2(Self::side(), Self::side()));
        self.paint_box(&painter, box_rect, state);
        if let Some(label) = &self.label {
            let text = UiText::resolve(painter.ctx(), label);
            let style = TextStyle::new(TypeSize::Label, FontFace::Regular);
            let color = <color::Text as ColorToken>::VALUE;
            let room = (rect.right() - gap - box_rect.right() - gap).max(0.0);
            let galley = Draw::galley(&painter, &text, style, color, TextFit::Truncate(room));
            let at = pos2(box_rect.right() + gap, rect.center().y - galley.size().y / 2.0);
            painter.galley(at, galley, color);
        }
    }

    fn paint_box(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        let on_accent = <color::OnAccent as ColorToken>::VALUE;
        match *self.value {
            Selection::Off => {
                Draw::fill(painter, rect, Corner::Small, StateStyle::Field.fill(state));
                Draw::stroke(painter, rect, Corner::Small, <color::LineStrong as ColorToken>::VALUE);
            }
            Selection::On => {
                Draw::fill(painter, rect, Corner::Small, StateStyle::Accent.fill(state));
                Icon::Check.paint(painter, rect, IconSize::Small, on_accent);
            }
            Selection::Mixed => {
                Draw::fill(painter, rect, Corner::Small, StateStyle::Accent.fill(state));
                let dash =
                    Rect::from_center_size(rect.center(), vec2(<spacing::S as SpacingToken>::VALUE, Self::DASH_HEIGHT));
                Draw::fill(painter, dash, Corner::Full, on_accent);
            }
        }
    }

    fn side() -> f32 {
        <spacing::L as SpacingToken>::VALUE
    }
}
