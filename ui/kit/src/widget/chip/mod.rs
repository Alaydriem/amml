mod chip_kind;
mod chip_size;
mod chip_tone;
mod key_cap;
mod key_cap_label;
mod key_cap_size;
mod key_chord;

pub use chip_kind::ChipKind;
pub use chip_size::ChipSize;
pub use chip_tone::ChipTone;
pub use key_cap::KeyCap;
pub use key_cap_label::KeyCapLabel;
pub use key_cap_size::KeyCapSize;
pub use key_chord::KeyChord;

use amml_i18n::text::Text;
use egui::{Color32, Context, CursorIcon, Painter, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::{Icon, IconSize};
use crate::state::{DisabledReason, InteractionState, StateStyle};
use crate::text::UiText;
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Chip + Chip style`. A round label: a filter the user clicks, or a
/// display chip in one tone, with an optional leading icon or dot. Row "HUD status" is the `ChipSize::Hud` size
/// with a dot.
pub struct Chip {
    kind: ChipKind,
    text: Text,
    icon: Option<Icon>,
    dot: Option<Color32>,
    size: ChipSize,
    disabled: Option<DisabledReason>,
    off: bool,
}

impl Chip {
    const STYLE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);

    pub fn new(kind: ChipKind, text: Text) -> Self {
        Self { kind, text, icon: None, dot: None, size: ChipSize::Regular, disabled: None, off: false }
    }

    pub fn size(mut self, size: ChipSize) -> Self {
        self.size = size;
        self
    }

    /// Leads with a `--spacing-dot` round dot in `tone`, a colour the caller reads from a colour token. A dot
    /// takes the place of the icon.
    pub fn dot(mut self, tone: Color32) -> Self {
        self.dot = Some(tone);
        self
    }

    /// The height of this chip's size.
    pub fn chip_height(&self) -> f32 {
        self.size.height()
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// The switched-off mod look (Paper: List row, "MOD OFF"): the text and icon in
    /// `--color-text-faint`, the fill kept. It is a data state, not the disabled state.
    pub fn off(mut self, off: bool) -> Self {
        self.off = off;
        self
    }

    /// The height every chip is.
    pub fn height() -> f32 {
        <spacing::ControlSmall as SpacingToken>::VALUE
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    pub fn desired_width(&self, ctx: &Context) -> f32 {
        let label = Draw::measure(ctx, &UiText::resolve(ctx, &self.text), Self::STYLE).x;
        (Self::padding() * 2.0 + self.lead_width() + label).ceil()
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let enabled = self.disabled.is_none();
        let clickable = matches!(self.kind, ChipKind::Filter { .. }) && enabled;
        let sense = if clickable { Sense::click() } else { Sense::hover() };
        // no wider than the room left: the label is cut with an ellipsis instead
        let height = self.chip_height();
        let width = self.desired_width(ui.ctx()).min(ui.available_width().max(height));
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), sense);
        self.paint(ui.painter(), rect, InteractionState::of(&response, enabled));
        match (&self.disabled, clickable) {
            (Some(reason), _) => reason.tooltip().attach(response),
            (None, true) => response.on_hover_cursor(CursorIcon::PointingHand),
            (None, false) => response,
        }
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        let painter = state.painter(painter);
        if state == InteractionState::Focused {
            FocusRing::paint(&painter, rect, Corner::Full.radius(rect));
        }
        let (fill, border, text) = self.colors(state);
        Draw::fill(&painter, rect, Corner::Full, fill);
        if let Some(border) = border {
            Draw::stroke(&painter, rect, Corner::Full, border);
        }
        let mut x = rect.left() + Self::padding();
        if let Some(tone) = self.dot {
            let dot = <spacing::Dot as SpacingToken>::VALUE;
            painter.circle_filled(pos2(x + dot / 2.0, rect.center().y), dot / 2.0, tone);
            x += dot + self.size.gap();
        } else if let Some(icon) = self.lead_icon() {
            let side = IconSize::Small.points();
            icon.paint(
                &painter,
                Rect::from_min_size(pos2(x, rect.center().y - side / 2.0), vec2(side, side)),
                IconSize::Small,
                text,
            );
            x += side + self.size.gap();
        }
        let label = UiText::resolve(painter.ctx(), &self.text);
        let room = (rect.right() - Self::padding() - x).max(0.0);
        let galley = Draw::galley(&painter, &label, Self::STYLE, text, TextFit::Truncate(room));
        painter.galley(pos2(x, rect.center().y - galley.size().y / 2.0), galley, text);
    }

    // (fill, border, text and icon)
    fn colors(&self, state: InteractionState) -> (Color32, Option<Color32>, Color32) {
        let (fill, border, text) = self.tone_colors(state);
        if self.off { (fill, border, <color::TextFaint as ColorToken>::VALUE) } else { (fill, border, text) }
    }

    // the kind's own (fill, border, text and icon)
    fn tone_colors(&self, state: InteractionState) -> (Color32, Option<Color32>, Color32) {
        match self.kind {
            ChipKind::Display(tone) => (tone.fill(), tone.border(), tone.text()),
            ChipKind::Filter { selected: true } => {
                (StateStyle::Soft.fill(state), None, <color::AccentHover as ColorToken>::VALUE)
            }
            ChipKind::Filter { selected: false } => {
                let text = match state {
                    InteractionState::Hover | InteractionState::Pressed | InteractionState::Focused => {
                        <color::Text as ColorToken>::VALUE
                    }
                    InteractionState::Rest | InteractionState::Disabled => <color::TextDim as ColorToken>::VALUE,
                };
                (StateStyle::Field.fill(state), None, text)
            }
        }
    }

    // a selected filter leads with a check; otherwise the chip's own icon, if any
    fn lead_icon(&self) -> Option<Icon> {
        match self.kind {
            ChipKind::Filter { selected: true } => Some(Icon::Check),
            ChipKind::Filter { selected: false } | ChipKind::Display(_) => self.icon,
        }
    }

    fn lead_width(&self) -> f32 {
        if self.dot.is_some() {
            return <spacing::Dot as SpacingToken>::VALUE + self.size.gap();
        }
        match self.lead_icon() {
            Some(_) => IconSize::Small.points() + self.size.gap(),
            None => 0.0,
        }
    }

    fn padding() -> f32 {
        <spacing::S as SpacingToken>::VALUE
    }
}
