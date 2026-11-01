mod colour_value;

pub use colour_value::ColourValue;

use amml_i18n::text::Text;
use egui::{Align2, Color32, CursorIcon, Id, Painter, Rect, Response, Sense, Ui, pos2, vec2};

use super::field::{FieldBox, FieldLook, InlineEdit, InlineOutcome};
use crate::AmmlUiMessage;
use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::id::AmmlUiId;
use crate::state::{DisabledReason, InteractionState};
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Mod manager, `Widget / Colour field`. A hex value and its swatch; a swatch with alpha is
/// split into a checker half and a solid half. Click to type a new hex.
pub struct ColourField<'a> {
    id: Id,
    value: &'a mut ColourValue,
    disabled: Option<DisabledReason>,
}

impl<'a> ColourField<'a> {
    /// Paper: Colour field sheet, the field padding left and right.
    const PADDING_LEFT: f32 = 10.0;
    const PADDING_RIGHT: f32 = 4.0;
    /// Paper: Colour field sheet, the swatch and its checker tile.
    const SWATCH_WIDTH: f32 = 52.0;
    const SWATCH_HEIGHT: f32 = 26.0;
    const CHECKER: f32 = 12.0;
    const STYLE: TextStyle = TextStyle::new(TypeSize::Body, FontFace::Regular);
    const REASON: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);

    pub fn new(id: Id, value: &'a mut ColourValue) -> Self {
        Self { id, value, disabled: None }
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    pub fn desired_width(&self, ctx: &egui::Context) -> f32 {
        // room for the longest hex, so the field does not change width while typing
        let text = Draw::measure(ctx, "#WWWWWWWW", Self::STYLE).x;
        (Self::PADDING_LEFT + text + <spacing::M as SpacingToken>::VALUE + Self::SWATCH_WIDTH + Self::PADDING_RIGHT)
            .ceil()
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let enabled = self.disabled.is_none();
        let error_id = self.id.with(AmmlUiId::FieldError);
        let refused = ui.data(|data| data.get_temp::<bool>(error_id)).unwrap_or(false);
        let height = <spacing::Button as SpacingToken>::VALUE;
        let reason = if refused { Self::gap() + Self::REASON.size.line_height() } else { 0.0 };
        let width = self.desired_width(ui.ctx());
        let (outer, _) = ui.allocate_exact_size(vec2(width, height + reason), Sense::hover());
        let rect = Rect::from_min_size(outer.min, vec2(width, height));
        let editing = InlineEdit::is_editing(ui, self.id);
        let sense = if enabled && !editing { Sense::click() } else { Sense::hover() };
        let mut response = ui.interact(rect, self.id, sense);
        if enabled && !editing && response.clicked() {
            InlineEdit::start(ui, self.id);
        }
        let state = if editing { InteractionState::Focused } else { InteractionState::of(&response, enabled) };
        let look = FieldLook { selection: None, invalid: refused.then(|| Text::message(AmmlUiMessage::InvalidColour)) };
        let painter = state.painter(ui.painter());
        self.paint_frame(&painter, rect, state, &look);
        let text_rect = Rect::from_min_max(
            pos2(rect.left() + Self::PADDING_LEFT, rect.top()),
            pos2(self.swatch_rect(rect).left(), rect.bottom()),
        );
        match InlineEdit::show(
            ui,
            self.id,
            text_rect,
            Self::STYLE,
            <color::Text as ColorToken>::VALUE,
            Align2::LEFT_CENTER,
            self.value.hex(),
        ) {
            InlineOutcome::Commit(typed) => match ColourValue::parse(&typed) {
                Some(value) => {
                    if value != *self.value {
                        *self.value = value;
                        response.mark_changed();
                    }
                    ui.data_mut(|data| data.remove::<bool>(error_id));
                }
                None => {
                    ui.data_mut(|data| data.insert_temp(error_id, true));
                }
            },
            InlineOutcome::Idle => self.paint_hex(&painter, rect, None),
            InlineOutcome::Editing | InlineOutcome::Cancel => {}
        }
        let response = if enabled && !editing { response.on_hover_cursor(CursorIcon::Text) } else { response };
        match &self.disabled {
            None => response,
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    /// Paints the field in `state` with `look`, for the gallery.
    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState, look: &FieldLook) {
        let painter = state.painter(painter);
        self.paint_frame(&painter, rect, state, look);
        self.paint_hex(&painter, rect, look.selection.clone());
    }

    // rest has no box; hover, editing and invalid show the field with a border
    fn paint_frame(&self, painter: &Painter, rect: Rect, state: InteractionState, look: &FieldLook) {
        let (fill, border) = match (&look.invalid, state) {
            (Some(_), _) => (<color::BgField as ColorToken>::VALUE, Some(<color::Danger as ColorToken>::VALUE)),
            (None, InteractionState::Focused) => {
                (<color::BgField as ColorToken>::VALUE, Some(<color::Accent as ColorToken>::VALUE))
            }
            (None, InteractionState::Hover | InteractionState::Pressed) => {
                (<color::BgField as ColorToken>::VALUE, Some(<color::LineStrong as ColorToken>::VALUE))
            }
            (None, InteractionState::Rest | InteractionState::Disabled) => (Color32::TRANSPARENT, None),
        };
        FieldBox::paint(painter, rect, fill, border, state == InteractionState::Focused);
        self.paint_swatch(painter, self.swatch_rect(rect));
        if let Some(reason) = &look.invalid {
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

    fn paint_hex(&self, painter: &Painter, rect: Rect, selection: Option<std::ops::Range<usize>>) {
        let text = <color::Text as ColorToken>::VALUE;
        let galley = Draw::galley(painter, &self.value.hex(), Self::STYLE, text, TextFit::Line);
        let at = pos2(rect.left() + Self::PADDING_LEFT, rect.center().y - galley.size().y / 2.0);
        FieldBox::paint_text(painter, at, galley, text, selection);
    }

    // with alpha: checker under the colour on the left half, the solid colour on the right half
    fn paint_swatch(&self, painter: &Painter, swatch: Rect) {
        let colour = self.value.color32();
        // from the stored bytes: Color32 is premultiplied, so its r/g/b are darkened by the alpha
        let [r, g, b, _] = self.value.rgba;
        let solid = Color32::from_rgb(r, g, b);
        if self.value.alpha {
            let half = Rect::from_min_size(swatch.min, vec2(swatch.width() / 2.0, swatch.height()));
            self.paint_checker(painter, half);
            painter.rect_filled(half, 0, colour);
            painter.rect_filled(Rect::from_min_max(pos2(half.right(), swatch.top()), swatch.max), 0, solid);
        } else {
            painter.rect_filled(swatch, 0, solid);
        }
        Draw::stroke(painter, swatch, Corner::Small, <color::LineStrong as ColorToken>::VALUE);
    }

    fn paint_checker(&self, painter: &Painter, area: Rect) {
        let painter = painter.with_clip_rect(area);
        painter.rect_filled(area, 0, <color::CheckerLight as ColorToken>::VALUE);
        let tile = Self::CHECKER / 2.0;
        let columns = (area.width() / tile).ceil() as usize;
        let rows = (area.height() / tile).ceil() as usize;
        for row in 0..rows {
            for column in (row % 2..columns).step_by(2) {
                let min = pos2(area.left() + column as f32 * tile, area.top() + row as f32 * tile);
                painter.rect_filled(
                    Rect::from_min_size(min, vec2(tile, tile)),
                    0,
                    <color::CheckerDark as ColorToken>::VALUE,
                );
            }
        }
    }

    fn swatch_rect(&self, rect: Rect) -> Rect {
        let right = rect.right() - Self::PADDING_RIGHT;
        Rect::from_min_size(
            pos2(right - Self::SWATCH_WIDTH, rect.center().y - Self::SWATCH_HEIGHT / 2.0),
            vec2(Self::SWATCH_WIDTH, Self::SWATCH_HEIGHT),
        )
    }

    fn gap() -> f32 {
        <spacing::S as SpacingToken>::VALUE
    }
}
