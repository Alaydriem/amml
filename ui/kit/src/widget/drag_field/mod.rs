mod drag_error;
mod drag_unit;

use drag_error::DragError;
pub use drag_unit::DragUnit;

use std::ops::RangeInclusive;

use amml_i18n::text::Text;
use egui::{Align2, Color32, CursorIcon, Id, Painter, Rect, Response, Sense, Ui, pos2, vec2};

use super::field::{FieldBox, FieldLook, InlineEdit, InlineOutcome};
use crate::AmmlUiMessage;
use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::id::AmmlUiId;
use crate::state::{DisabledReason, InteractionState};
use crate::text::UiText;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Drag field`. A number you drag left or right, or click to type; with
/// an optional label in a role colour and a unit.
pub struct DragField<'a> {
    id: Id,
    value: &'a mut f64,
    label: Option<(Text, Color32)>,
    unit: DragUnit,
    decimals: u8,
    step: f64,
    range: Option<(RangeInclusive<f64>, Text)>,
    disabled: Option<DisabledReason>,
}

impl<'a> DragField<'a> {
    const VALUE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);
    const LABEL: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold);
    const REASON: TextStyle = TextStyle::new(TypeSize::Micro, FontFace::Regular);

    pub fn new(id: Id, value: &'a mut f64) -> Self {
        Self { id, value, label: None, unit: DragUnit::None, decimals: 2, step: 0.1, range: None, disabled: None }
    }

    /// A label at the left, in a colour the caller picks from a token selector (axis or track).
    pub fn label(mut self, text: Text, color: Color32) -> Self {
        self.label = Some((text, color));
        self
    }

    pub fn unit(mut self, unit: DragUnit) -> Self {
        self.unit = unit;
        self
    }

    pub fn decimals(mut self, decimals: u8) -> Self {
        self.decimals = decimals;
        self
    }

    /// The value change per point dragged.
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Clamps dragging to `range`; a typed value outside it shows `reason`.
    pub fn range(mut self, range: RangeInclusive<f64>, reason: Text) -> Self {
        self.range = Some((range, reason));
        self
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    pub fn show(self, ui: &mut Ui, width: f32) -> Response {
        let enabled = self.disabled.is_none();
        let error_id = self.id.with(AmmlUiId::FieldError);
        let error = ui.data(|data| data.get_temp::<DragError>(error_id));
        let reason_height = if error.is_some() { Self::gap() + Self::REASON.size.line_height() } else { 0.0 };
        let height = <spacing::Field as SpacingToken>::VALUE;
        let (outer, _) = ui.allocate_exact_size(vec2(width, height + reason_height), Sense::hover());
        let rect = Rect::from_min_size(outer.min, vec2(width, height));
        let editing = InlineEdit::is_editing(ui, self.id);
        let sense = if enabled && !editing { Sense::click_and_drag() } else { Sense::hover() };
        let mut response = ui.interact(rect, self.id, sense);
        if enabled && !editing {
            if response.dragged() {
                let moved = self.clamp(*self.value + f64::from(response.drag_delta().x) * self.step);
                if (moved - *self.value).abs() > f64::EPSILON {
                    *self.value = moved;
                    response.mark_changed();
                    ui.data_mut(|data| data.remove::<DragError>(error_id));
                }
            } else if response.clicked() {
                InlineEdit::start(ui, self.id);
            }
        }
        let state = if editing { InteractionState::Focused } else { InteractionState::of(&response, enabled) };
        let invalid = error.map(|error| self.reason(error));
        let look = FieldLook { selection: None, invalid };
        let painter = state.painter(ui.painter());
        self.paint_frame(&painter, rect, state, &look);
        let style_color = <color::Text as ColorToken>::VALUE;
        match InlineEdit::show(
            ui,
            self.id,
            self.value_rect(ui.ctx(), rect),
            Self::VALUE,
            style_color,
            Align2::RIGHT_CENTER,
            self.number(),
        ) {
            InlineOutcome::Commit(typed) => match self.check(&typed) {
                Ok(value) => {
                    if (value - *self.value).abs() > f64::EPSILON {
                        response.mark_changed();
                    }
                    *self.value = value;
                    ui.data_mut(|data| data.remove::<DragError>(error_id));
                }
                Err(error) => {
                    ui.data_mut(|data| data.insert_temp(error_id, error));
                }
            },
            InlineOutcome::Idle => self.paint_value(&painter, rect, None),
            InlineOutcome::Editing | InlineOutcome::Cancel => {}
        }
        let response =
            if enabled && !editing { response.on_hover_cursor(CursorIcon::ResizeHorizontal) } else { response };
        match &self.disabled {
            None => response,
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    /// Paints the field in `state` with `look`, for the gallery; Pressed is the dragging look.
    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState, look: &FieldLook) {
        let painter = state.painter(painter);
        self.paint_frame(&painter, rect, state, look);
        self.paint_value(&painter, rect, look.selection.clone());
    }

    fn paint_frame(&self, painter: &Painter, rect: Rect, state: InteractionState, look: &FieldLook) {
        let fill = match state {
            InteractionState::Hover => <color::BgHover as ColorToken>::VALUE,
            InteractionState::Pressed => <color::BgActive as ColorToken>::VALUE,
            _ => <color::BgField as ColorToken>::VALUE,
        };
        let border = match (&look.invalid, state) {
            (Some(_), _) => Some(<color::Danger as ColorToken>::VALUE),
            (None, InteractionState::Pressed) => Some(<color::Accent as ColorToken>::VALUE),
            (None, _) => None,
        };
        FieldBox::paint(painter, rect, fill, border, state == InteractionState::Focused);
        if let Some((text, color)) = &self.label {
            let label = UiText::resolve(painter.ctx(), text);
            let galley = Draw::galley(painter, &label, Self::LABEL, *color, TextFit::Line);
            painter.galley(
                pos2(rect.left() + Self::padding(), rect.center().y - galley.size().y / 2.0),
                galley,
                *color,
            );
        }
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

    // the number and its unit, right-aligned
    fn paint_value(&self, painter: &Painter, rect: Rect, selection: Option<std::ops::Range<usize>>) {
        let number = self.number();
        let text = <color::Text as ColorToken>::VALUE;
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let unit = match &self.unit {
            DragUnit::None => None,
            DragUnit::Degrees => Some(("°".to_owned(), 0.0)),
            DragUnit::Text(word) => Some((UiText::resolve(painter.ctx(), word), <spacing::Xs as SpacingToken>::VALUE)),
            DragUnit::Joined(unit) => Some((UiText::resolve(painter.ctx(), unit), 0.0)),
        };
        let mut right = rect.right() - Self::padding();
        if let Some((unit, gap)) = unit {
            let galley = Draw::galley(painter, &unit, Self::VALUE, faint, TextFit::Line);
            right -= galley.size().x;
            painter.galley(pos2(right, rect.center().y - galley.size().y / 2.0), galley, faint);
            right -= gap;
        }
        // a very long number is cut before it reaches the label
        let room = (right - self.value_rect(painter.ctx(), rect).left()).max(0.0);
        let galley = Draw::galley(painter, &number, Self::VALUE, text, TextFit::Truncate(room));
        let at = pos2(right - galley.size().x, rect.center().y - galley.size().y / 2.0);
        FieldBox::paint_text(painter, at, galley, text, selection);
    }

    // where the typed text goes: right of the label, up to the padding
    fn value_rect(&self, ctx: &egui::Context, rect: Rect) -> Rect {
        let label = self
            .label
            .as_ref()
            .map_or(0.0, |(text, _)| Draw::measure(ctx, &UiText::resolve(ctx, text), Self::LABEL).x + Self::gap());
        Rect::from_min_max(
            pos2(rect.left() + Self::padding() + label, rect.top()),
            pos2(rect.right() - Self::padding(), rect.bottom()),
        )
    }

    fn number(&self) -> String {
        format!("{:.*}", usize::from(self.decimals), self.current())
    }

    fn check(&self, typed: &str) -> Result<f64, DragError> {
        let value = typed.trim().parse::<f64>().ok().filter(|value| value.is_finite()).ok_or(DragError::NotANumber)?;
        match &self.range {
            Some((range, _)) if !range.contains(&value) => Err(DragError::OutOfRange),
            _ => Ok(value),
        }
    }

    fn reason(&self, error: DragError) -> Text {
        match (error, &self.range) {
            (DragError::OutOfRange, Some((_, reason))) => reason.clone(),
            _ => Text::message(AmmlUiMessage::EnterANumber),
        }
    }

    fn clamp(&self, value: f64) -> f64 {
        match &self.range {
            Some((range, _)) => value.clamp(*range.start(), range.end().max(*range.start())),
            None => value,
        }
    }

    fn current(&self) -> f64 {
        if self.value.is_finite() { *self.value } else { 0.0 }
    }

    fn padding() -> f32 {
        <spacing::S as SpacingToken>::VALUE
    }

    fn gap() -> f32 {
        <spacing::Xs as SpacingToken>::VALUE
    }
}
