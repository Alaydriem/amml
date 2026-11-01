use amml_i18n::text::{Text, UserText};
use egui::{Color32, CursorIcon, Painter, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::{Icon, IconSize};
use crate::state::{DisabledReason, InteractionState, StateStyle};
use crate::text::UiText;
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{Chip, ChipKind, ChipTone};

/// Paper: Mod manager, `Widget / Nav entry`. A page in a side list: an icon, a label, and a
/// warning count or a plain tally at the right when there is one. The whole row is the click target.
pub struct NavEntry {
    icon: Icon,
    text: Text,
    count: u32,
    tally: Option<u32>,
    selected: bool,
    disabled: Option<DisabledReason>,
}

impl NavEntry {
    /// Paper: Nav entry, the row height.
    pub const ROW: f32 = 48.0;
    const LABEL: TextStyle = TextStyle::new(TypeSize::Body, FontFace::Regular);
    const TALLY: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);

    pub fn new(icon: Icon, text: Text) -> Self {
        Self { icon, text, count: 0, tally: None, selected: false, disabled: None }
    }

    /// A warning chip with `count` at the right, shown when it is above 0. With a count the icon
    /// is WARNING in `--color-warning`.
    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    /// A neutral number at the right, `--text-note` in `--color-text-faint`, drawn for every value
    /// including 0. The icon does not change. A `count` above 0 replaces it.
    pub fn tally(mut self, tally: u32) -> Self {
        self.tally = Some(tally);
        self
    }

    /// The open page: `--color-bg-active` and the accent bar at the left.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let enabled = self.disabled.is_none();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::ROW), sense);
        self.paint(ui.painter(), rect, InteractionState::of(&response, enabled));
        match &self.disabled {
            None => response.on_hover_cursor(CursorIcon::PointingHand),
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        let painter = state.painter(painter);
        if state == InteractionState::Focused {
            FocusRing::paint(&painter, rect, Corner::Panel.radius(rect));
        }
        let fill = if self.selected { <color::BgActive as ColorToken>::VALUE } else { StateStyle::Clear.fill(state) };
        Draw::fill(&painter, rect, Corner::Panel, fill);
        if self.selected {
            let height = rect.height() - 2.0 * <spacing::M as SpacingToken>::VALUE;
            let bar = Rect::from_min_size(
                pos2(rect.left(), rect.center().y - height / 2.0),
                vec2(<spacing::Indicator as SpacingToken>::VALUE, height),
            );
            Draw::fill(&painter, bar, Corner::Full, <color::Accent as ColorToken>::VALUE);
        }

        let pad = <spacing::L as SpacingToken>::VALUE;
        let gap = <spacing::M as SpacingToken>::VALUE;
        let slot_side = <spacing::ControlSmall as SpacingToken>::VALUE;
        let slot =
            Rect::from_min_size(pos2(rect.left() + pad, rect.center().y - slot_side / 2.0), vec2(slot_side, slot_side));
        let (icon, icon_colour) = self.icon_look(state.inner());
        icon.paint(&painter, slot, IconSize::Regular, icon_colour);

        let mut right = rect.right() - pad;
        if self.count > 0 {
            let chip = Chip::new(
                ChipKind::Display(ChipTone::Warning),
                Text::user(UserText::from_model(self.count.to_string())),
            );
            let size = vec2(chip.desired_width(painter.ctx()), Chip::height());
            let place = Rect::from_min_size(pos2(right - size.x, rect.center().y - size.y / 2.0), size);
            chip.paint(&painter, place, InteractionState::Rest);
            right = place.left() - gap;
        } else if let Some(tally) = self.tally {
            let faint = <color::TextFaint as ColorToken>::VALUE;
            let number = UiText::resolve(painter.ctx(), &Text::user(UserText::from_model(tally.to_string())));
            let galley = Draw::galley(&painter, &number, Self::TALLY, faint, TextFit::Line);
            let origin = pos2(right - galley.size().x, rect.center().y - galley.size().y / 2.0);
            right = origin.x - gap;
            painter.galley(origin, galley, faint);
        }
        let left = slot.right() + gap;
        let text = <color::Text as ColorToken>::VALUE;
        let label = UiText::resolve(painter.ctx(), &self.text);
        let galley = Draw::galley(&painter, &label, Self::LABEL, text, TextFit::Truncate((right - left).max(0.0)));
        painter.galley(pos2(left, rect.center().y - galley.size().y / 2.0), galley, text);
    }

    // (icon, colour): a count turns the icon to WARNING; hover and press brighten it
    fn icon_look(&self, state: InteractionState) -> (Icon, Color32) {
        if self.count > 0 {
            return (Icon::Warning, <color::Warning as ColorToken>::VALUE);
        }
        let colour = match state {
            InteractionState::Hover | InteractionState::Pressed => <color::Text as ColorToken>::VALUE,
            InteractionState::Rest | InteractionState::Focused | InteractionState::Disabled => {
                <color::TextDim as ColorToken>::VALUE
            }
        };
        (self.icon, colour)
    }
}
