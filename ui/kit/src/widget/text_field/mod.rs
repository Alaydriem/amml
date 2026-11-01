mod text_field_kind;

pub use text_field_kind::TextFieldKind;

use amml_i18n::text::Text;
use egui::{Color32, Frame, Id, Key, Painter, Rect, Response, RichText, Sense, TextEdit, Ui, pos2, vec2};

use super::field::{FieldBox, FieldLook};
use crate::AmmlUiMessage;
use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::{Icon, IconSize};
use crate::id::AmmlUiId;
use crate::state::{DisabledReason, InteractionState};
use crate::text::UiText;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{IconButton, IconButtonSize};

/// Paper: Widgets, `Widget / Text field`. One line of text in a `--color-bg-field` box; the search
/// kind leads with an icon and ends with a clear button while there is text. With a leading swatch
/// (Paper state `Leading swatch`) it is a name field: button high, the name in label size, semibold.
pub struct TextField<'a> {
    id: Id,
    value: &'a mut String,
    kind: TextFieldKind,
    placeholder: Text,
    invalid: Option<Text>,
    disabled: Option<DisabledReason>,
    swatch: Option<Color32>,
}

impl<'a> TextField<'a> {
    const STYLE: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);
    const NAME_STYLE: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Semibold);
    const REASON: TextStyle = TextStyle::new(TypeSize::Micro, FontFace::Regular);
    /// Paper: Text field sheet, the leading swatch side.
    const SWATCH: f32 = 12.0;
    /// Paper: Text field sheet, the gap between the leading swatch and the name.
    const SWATCH_GAP: f32 = 10.0;
    /// Paper: Text field sheet, how much of `--color-line-strong` the name field's border shows.
    const NAME_BORDER_MIX: f32 = 0.7;

    /// The placeholder shows while the field is empty, and is the field's accessible name.
    pub fn new(id: Id, value: &'a mut String, kind: TextFieldKind, placeholder: Text) -> Self {
        Self { id, value, kind, placeholder, invalid: None, disabled: None, swatch: None }
    }

    /// A colour square before the text; the field becomes a name field.
    pub fn leading_swatch(mut self, colour: Color32) -> Self {
        self.swatch = Some(colour);
        self
    }

    /// The danger border and this reason under the field.
    pub fn invalid(mut self, reason: Option<Text>) -> Self {
        self.invalid = reason;
        self
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    pub fn show(self, ui: &mut Ui, width: f32) -> Response {
        let enabled = self.disabled.is_none();
        let rect = self.allocate(ui, width);
        let box_response = ui.interact(rect, self.id.with(AmmlUiId::FieldBox), Sense::hover());
        let focused = ui.memory(|memory| memory.has_focus(self.id));
        let state = if !enabled {
            InteractionState::Disabled
        } else if focused {
            InteractionState::Focused
        } else if box_response.hovered() {
            InteractionState::Hover
        } else {
            InteractionState::Rest
        };
        let painter = state.painter(ui.painter());
        self.paint_frame(&painter, rect, state, self.invalid.as_ref());
        let start_id = self.id.with(AmmlUiId::FieldStart);
        let placeholder = UiText::resolve(ui.ctx(), &self.placeholder);
        let font = self.style().font_id();
        let hint = RichText::new(placeholder).font(font.clone()).color(<color::TextFaint as ColorToken>::VALUE);
        let text_rect = self.text_rect(rect);
        let edit = TextEdit::singleline(&mut *self.value)
            .id(self.id)
            .frame(Frame::NONE)
            .margin(egui::Margin::ZERO)
            .font(font)
            .text_color(<color::Text as ColorToken>::VALUE)
            .hint_text(hint)
            .desired_width(text_rect.width());
        let mut response = ui.add_enabled_ui(enabled, |ui| ui.put(text_rect, edit)).inner;
        if response.gained_focus() {
            ui.data_mut(|data| data.insert_temp(start_id, self.value.clone()));
        }
        // Escape takes the focus away and puts back the value the field had when it took the focus
        let escaped = response.lost_focus() && ui.input(|input| input.key_pressed(Key::Escape));
        if escaped
            && let Some(start) = ui.data(|data| data.get_temp::<String>(start_id))
            && *self.value != start
        {
            *self.value = start;
            response.mark_changed();
        }
        if self.kind == TextFieldKind::Search && !self.value.is_empty() && enabled {
            let clear = IconButton::new(Icon::Close)
                .size(IconButtonSize::Small)
                .tooltip(Text::message(AmmlUiMessage::ClearSearch));
            if ui.put(self.clear_rect(rect), |ui: &mut Ui| clear.show(ui)).clicked() {
                self.value.clear();
                response.mark_changed();
                ui.memory_mut(|memory| memory.request_focus(self.id));
            }
        }
        let response = response.union(box_response);
        match &self.disabled {
            None => response,
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    /// Paints the field in `state` with `look` (typed selection, invalid reason), for the gallery.
    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState, look: &FieldLook) {
        let painter = state.painter(painter);
        self.paint_frame(&painter, rect, state, look.invalid.as_ref().or(self.invalid.as_ref()));
        let text_rect = self.text_rect(rect);
        let empty = self.value.is_empty();
        let (shown, colour) = if empty {
            (UiText::resolve(painter.ctx(), &self.placeholder), <color::TextFaint as ColorToken>::VALUE)
        } else {
            (self.value.clone(), <color::Text as ColorToken>::VALUE)
        };
        let galley = Draw::galley(&painter, &shown, self.style(), colour, TextFit::Truncate(text_rect.width()));
        let at = pos2(text_rect.left(), text_rect.center().y - galley.size().y / 2.0);
        FieldBox::paint_text(&painter, at, galley, colour, if empty { None } else { look.selection.clone() });
        if self.kind == TextFieldKind::Search && !empty {
            let clear = IconButton::new(Icon::Close).size(IconButtonSize::Small);
            clear.paint(&painter, self.clear_rect(rect), InteractionState::Rest);
        }
    }

    // the box, the ring, the danger border, the search icon and the reason line
    fn paint_frame(&self, painter: &Painter, rect: Rect, state: InteractionState, invalid: Option<&Text>) {
        let fill = match state {
            InteractionState::Hover => <color::BgHover as ColorToken>::VALUE,
            _ => <color::BgField as ColorToken>::VALUE,
        };
        let border = match (invalid, self.swatch) {
            (Some(_), _) => Some(<color::Danger as ColorToken>::VALUE),
            (None, Some(_)) => Some(<color::LineStrong as ColorToken>::VALUE.gamma_multiply(Self::NAME_BORDER_MIX)),
            (None, None) => None,
        };
        FieldBox::paint(painter, rect, fill, border, state == InteractionState::Focused);
        if let Some(colour) = self.swatch {
            let side = Self::SWATCH;
            let at = pos2(rect.left() + Self::padding(), rect.center().y - side / 2.0);
            Draw::fill(painter, Rect::from_min_size(at, vec2(side, side)), Corner::Small, colour);
        }
        if self.kind == TextFieldKind::Search {
            let side = IconSize::Regular.points();
            let slot = Rect::from_min_size(
                pos2(rect.left() + Self::padding(), rect.center().y - side / 2.0),
                vec2(side, side),
            );
            Icon::Search.paint(painter, slot, IconSize::Regular, <color::TextFaint as ColorToken>::VALUE);
        }
        if let Some(reason) = invalid {
            FieldBox::paint_reason(
                painter,
                rect,
                Self::gap(),
                reason,
                Self::REASON,
                <color::Danger as ColorToken>::VALUE,
            );
        }
    }

    // the whole field: the box, then the reason line when invalid
    fn allocate(&self, ui: &mut Ui, width: f32) -> Rect {
        let field = match self.swatch {
            Some(_) => <spacing::Button as SpacingToken>::VALUE,
            None => <spacing::Field as SpacingToken>::VALUE,
        };
        let reason = if self.invalid.is_some() { Self::gap() + Self::REASON.size.line_height() } else { 0.0 };
        let (outer, _) = ui.allocate_exact_size(vec2(width, field + reason), Sense::hover());
        Rect::from_min_size(outer.min, vec2(width, field))
    }

    // where the text goes: after the search icon and before the clear slot
    fn text_rect(&self, rect: Rect) -> Rect {
        let mut left = rect.left() + Self::padding();
        let mut right = rect.right() - Self::padding();
        if self.kind == TextFieldKind::Search {
            left += IconSize::Regular.points() + Self::gap();
            right = self.clear_rect(rect).left() - Self::gap();
        }
        if self.swatch.is_some() {
            left += Self::SWATCH + Self::SWATCH_GAP;
        }
        let row = self.style().size.line_height();
        let top = rect.center().y - row / 2.0;
        Rect::from_min_max(pos2(left, top), pos2(right.max(left), top + row))
    }

    // the clear button slot, kept empty when there is no text so the width does not jump
    fn clear_rect(&self, rect: Rect) -> Rect {
        let side = IconButtonSize::Small.side();
        let right = rect.right() - Self::gap();
        Rect::from_min_size(pos2(right - side, rect.center().y - side / 2.0), vec2(side, side))
    }

    fn style(&self) -> TextStyle {
        if self.swatch.is_some() { Self::NAME_STYLE } else { Self::STYLE }
    }

    fn padding() -> f32 {
        <spacing::M as SpacingToken>::VALUE
    }

    fn gap() -> f32 {
        <spacing::S as SpacingToken>::VALUE
    }
}
