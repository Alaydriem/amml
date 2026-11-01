use amml_i18n::text::Text;
use egui::{ClosableTag, Color32, CursorIcon, Painter, Rect, Response, Sense, Stroke, Ui, pos2, vec2};

use super::{MenuLeading, MenuTone, MenuTrailing};
use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::glyph::SwatchDot;
use crate::icon::{Icon, IconSize};
use crate::state::{DisabledReason, InteractionState, StateStyle};
use crate::text::UiText;
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Menu item`. One row of a menu: a leading slot, the label and a trailing
/// part. A disabled item shows its reason as a second line, at full opacity.
pub struct MenuItem {
    text: Text,
    leading: MenuLeading,
    trailing: MenuTrailing,
    tone: MenuTone,
    disabled: Option<DisabledReason>,
}

impl MenuItem {
    /// Paper: Menu item sheet, the height of a disabled item with its reason line.
    pub const REASON_HEIGHT: f32 = 44.0;
    const LABEL: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);
    const SHORTCUT: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);
    const REASON: TextStyle = TextStyle::new(TypeSize::Micro, FontFace::Regular);
    /// Paper: Menu item sheet, the radio mark's outside diameter.
    const RADIO: f32 = 12.0;

    pub fn new(text: Text) -> Self {
        Self { text, leading: MenuLeading::None, trailing: MenuTrailing::None, tone: MenuTone::Normal, disabled: None }
    }

    pub fn leading(mut self, leading: MenuLeading) -> Self {
        self.leading = leading;
        self
    }

    pub fn trailing(mut self, trailing: MenuTrailing) -> Self {
        self.trailing = trailing;
        self
    }

    pub fn tone(mut self, tone: MenuTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    /// Shows the row the menu's width. A click on an enabled leaf (not a submenu row) closes the menu it is in; a row
    /// outside a menu (a list built from menu rows) closes nothing.
    pub fn show(self, ui: &mut Ui) -> Response {
        let enabled = self.disabled.is_none();
        let height = if enabled { <spacing::Row as SpacingToken>::VALUE } else { Self::REASON_HEIGHT };
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), height), sense);
        self.paint(ui.painter(), rect, InteractionState::of(&response, enabled));
        if response.clicked() && !matches!(self.trailing, MenuTrailing::Submenu) && Self::in_closable(ui) {
            ui.close();
        }
        if enabled { response.on_hover_cursor(CursorIcon::PointingHand) } else { response }
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        if state == InteractionState::Focused {
            FocusRing::paint(painter, rect, Corner::Control.radius(rect));
        }
        Draw::fill(painter, rect, Corner::Control, StateStyle::Clear.fill(state.inner()));
        // disabled mutes the icon and the label only; the reason stays readable
        let muted = state.painter(painter);
        let padding = <spacing::S as SpacingToken>::VALUE;
        let slot = IconSize::Regular.points();
        let label_top = match &self.disabled {
            Some(_) => rect.center().y - (Self::LABEL.size.line_height() + Self::REASON.size.line_height()) / 2.0,
            None => rect.center().y - Self::LABEL.size.line_height() / 2.0,
        };
        let line_centre = label_top + Self::LABEL.size.line_height() / 2.0;
        let slot_rect = Rect::from_center_size(pos2(rect.left() + padding + slot / 2.0, line_centre), vec2(slot, slot));
        self.paint_leading(&muted, slot_rect);
        let trailing_left = self.paint_trailing(&muted, rect, line_centre);
        let left = slot_rect.right() + padding;
        let room = (trailing_left - padding - left).max(0.0);
        let label = UiText::resolve(painter.ctx(), &self.text);
        let galley = Draw::galley(&muted, &label, Self::LABEL, self.label_color(), TextFit::Truncate(room));
        muted.galley(pos2(left, label_top), galley, self.label_color());
        if let Some(reason) = &self.disabled {
            let text = UiText::resolve(painter.ctx(), reason.text());
            let faint = <color::TextFaint as ColorToken>::VALUE;
            let galley = Draw::galley(painter, &text, Self::REASON, faint, TextFit::Truncate(room));
            painter.galley(pos2(left, label_top + Self::LABEL.size.line_height()), galley, faint);
        }
    }

    fn paint_leading(&self, painter: &Painter, slot: Rect) {
        match self.leading {
            MenuLeading::None => {}
            MenuLeading::Icon(icon) => icon.paint(painter, slot, IconSize::Regular, self.icon_color()),
            MenuLeading::Check => {
                Icon::Check.paint(painter, slot, IconSize::Regular, <color::AccentHover as ColorToken>::VALUE);
            }
            MenuLeading::Swatch(colour) => SwatchDot::paint(painter, slot, colour),
            MenuLeading::Radio(chosen) => Self::paint_radio(painter, slot, chosen),
        }
    }

    // the ring is `RADIO` across outside; the stroke sits on the circle line, so the radius loses half of it
    fn paint_radio(painter: &Painter, slot: Rect, chosen: bool) {
        let width = <spacing::StrokeThin as SpacingToken>::VALUE;
        let ring = if chosen { <color::Accent as ColorToken>::VALUE } else { <color::TextDim as ColorToken>::VALUE };
        painter.circle_stroke(slot.center(), (Self::RADIO - width) / 2.0, Stroke::new(width, ring));
        if chosen {
            let dot = <spacing::Dot as SpacingToken>::VALUE;
            painter.circle_filled(slot.center(), dot / 2.0, <color::Accent as ColorToken>::VALUE);
        }
    }

    // paints the trailing part at the right and returns its left edge
    fn paint_trailing(&self, painter: &Painter, rect: Rect, centre_y: f32) -> f32 {
        let right = rect.right() - <spacing::S as SpacingToken>::VALUE;
        let small = IconSize::Small.points();
        let icon_slot = |painter: &Painter, icon: Icon, colour: Color32| {
            let slot = Rect::from_center_size(pos2(right - small / 2.0, centre_y), vec2(small, small));
            icon.paint(painter, slot, IconSize::Small, colour);
            slot.left()
        };
        match &self.trailing {
            MenuTrailing::None => right,
            MenuTrailing::Shortcut(text) => {
                let faint = <color::TextFaint as ColorToken>::VALUE;
                let shortcut = UiText::resolve(painter.ctx(), text);
                let galley = Draw::galley(painter, &shortcut, Self::SHORTCUT, faint, TextFit::Line);
                let at = pos2(right - galley.size().x, centre_y - galley.size().y / 2.0);
                painter.galley(at, galley, faint);
                at.x
            }
            MenuTrailing::Submenu => icon_slot(painter, Icon::Chevron, <color::TextDim as ColorToken>::VALUE),
            MenuTrailing::Check => icon_slot(painter, Icon::Check, <color::AccentHover as ColorToken>::VALUE),
        }
    }

    // egui warns when `close` finds no menu or popup above the row
    fn in_closable(ui: &Ui) -> bool {
        ui.stack().iter().any(|stack| stack.info.tags.get_downcast::<ClosableTag>(ClosableTag::NAME).is_some())
    }

    fn label_color(&self) -> Color32 {
        match self.tone {
            MenuTone::Normal => <color::Text as ColorToken>::VALUE,
            MenuTone::Danger => <color::Danger as ColorToken>::VALUE,
        }
    }

    fn icon_color(&self) -> Color32 {
        match self.tone {
            MenuTone::Normal => <color::TextDim as ColorToken>::VALUE,
            MenuTone::Danger => <color::Danger as ColorToken>::VALUE,
        }
    }
}
