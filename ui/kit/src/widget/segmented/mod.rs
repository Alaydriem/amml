mod segment;

pub use segment::Segment;

use egui::{Color32, CursorIcon, Id, Key, Painter, Rect, Response, Sense, Stroke, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::IconSize;
use crate::id::AmmlUiId;
use crate::motion::Motion;
use crate::state::{ArrowKeys, DisabledReason, InteractionState, StateStyle};
use crate::text::UiText;
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Segmented`. Segments in a `--color-bg-field` track; the selected one
/// has the soft fill and slides when the choice changes. With `columns` (Paper row `Grid, two rows`) the
/// segments are separate buttons in a grid, and the selected one has the accent fill.
pub struct Segmented<'a> {
    id: Id,
    selected: &'a mut usize,
    segments: Vec<Segment>,
    disabled: Option<DisabledReason>,
    /// Segments that cannot be chosen, each with its reason; the others still work.
    disabled_segments: Vec<(usize, DisabledReason)>,
    columns: Option<usize>,
    /// The height of the track, when not the field height.
    height: Option<f32>,
}

impl<'a> Segmented<'a> {
    /// Paper: Segmented sheet, the inset between the track and the segments.
    const INSET: f32 = 2.0;
    /// Paper: Segmented sheet, the divider height.
    const DIVIDER_HEIGHT: f32 = 14.0;
    /// Paper: Segmented sheet, the gap between the buttons of the grid.
    const GRID_GAP: f32 = 5.0;
    const GRID_LABEL: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);
    const GRID_LABEL_SELECTED: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Semibold);

    pub fn new(id: Id, selected: &'a mut usize, segments: Vec<Segment>) -> Self {
        Self { id, selected, segments, disabled: None, disabled_segments: Vec::new(), columns: None, height: None }
    }

    /// Lays the segments out as a grid of `columns` buttons per row, `--spacing-row` high and 5 apart.
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = Some(columns.max(1));
        self
    }

    /// Makes the track `height` high instead of `--spacing-field` (Paper `338-0`: `--spacing-row`, 28). A grid keeps
    /// its own height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    /// Makes segment `index` one that cannot be chosen: it paints disabled and shows `reason` as its tooltip.
    pub fn disable_segment(mut self, index: usize, reason: DisabledReason) -> Self {
        self.disabled_segments.push((index, reason));
        self
    }

    /// Shows the control `width` wide; an all-icon control hugs its segments instead.
    pub fn show(self, ui: &mut Ui, width: f32) -> Response {
        let enabled = self.disabled.is_none();
        let width = if self.all_icons() { self.icon_width() } else { width };
        let (rect, mut response) = ui.allocate_exact_size(vec2(width, self.track_height()), Sense::hover());
        let cells = self.cells(rect);
        let mut states = Vec::with_capacity(cells.len());
        let mut focused = None;
        for (index, cell) in cells.iter().enumerate() {
            let reason = self.segment_reason(index).cloned();
            let open = enabled && reason.is_none();
            let sense = if open { Sense::click() } else { Sense::hover() };
            let segment = ui.interact(*cell, self.id.with(index), sense);
            if segment.clicked() && *self.selected != index {
                *self.selected = index;
                response.mark_changed();
            }
            if segment.has_focus() {
                ArrowKeys::keep(ui, segment.id);
                focused = Some(index);
            }
            states.push(InteractionState::of(&segment, open));
            match reason {
                Some(reason) if enabled => {
                    reason.tooltip().attach(segment);
                }
                _ if open => response |= segment.on_hover_cursor(CursorIcon::PointingHand),
                _ => {}
            }
        }
        if let Some(index) = focused {
            let step = ui.input(|input| {
                i32::from(input.key_pressed(Key::ArrowRight)) - i32::from(input.key_pressed(Key::ArrowLeft))
            });
            let target = (index as i64 + i64::from(step)).clamp(0, cells.len() as i64 - 1) as usize;
            if step != 0 && target != *self.selected && self.segment_reason(target).is_none() {
                *self.selected = target;
                response.mark_changed();
                ui.memory_mut(|memory| memory.request_focus(self.id.with(target)));
            }
        }
        let target_left = cells.get(*self.selected).map_or(rect.left(), |cell| cell.left());
        let slide = Motion::value(ui.ctx(), self.id.with(AmmlUiId::SegmentSlide), target_left);
        self.paint_with(ui.painter(), rect, &states, slide);
        match &self.disabled {
            None => response,
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    /// Paints the control with `state` on segment `target` and the others at rest, for the gallery.
    /// Disabled applies to the whole control; a segment made with `disable_segment` paints disabled in every state.
    pub fn paint(&self, painter: &Painter, rect: Rect, target: usize, state: InteractionState) {
        let cells = self.cells(rect);
        let states: Vec<_> = (0..cells.len())
            .map(|index| match state {
                InteractionState::Disabled => InteractionState::Disabled,
                _ if self.segment_reason(index).is_some() => InteractionState::Disabled,
                _ if index == target => state,
                _ => InteractionState::Rest,
            })
            .collect();
        let left = cells.get(*self.selected).map_or(rect.left(), |cell| cell.left());
        self.paint_with(painter, rect, &states, left);
    }

    /// The width an all-icon control takes.
    pub fn icon_width(&self) -> f32 {
        self.segments.len() as f32 * <spacing::Button as SpacingToken>::VALUE + Self::INSET * 2.0
    }

    fn paint_with(&self, painter: &Painter, rect: Rect, states: &[InteractionState], slide_left: f32) {
        if self.columns.is_some() {
            self.paint_grid(painter, rect, states);
            return;
        }
        let disabled = !states.is_empty() && states.iter().all(|state| *state == InteractionState::Disabled);
        let painter = if disabled { InteractionState::Disabled.painter(painter) } else { painter.clone() };
        let cells = self.cells(rect);
        let selected = *self.selected;
        Draw::fill(&painter, rect, Corner::Control, <color::BgField as ColorToken>::VALUE);
        // the ring goes under the focused segment, which then repaints the track behind it
        for (index, cell) in cells.iter().enumerate() {
            if states.get(index) == Some(&InteractionState::Focused) {
                FocusRing::paint(&painter, *cell, Corner::Small.radius(*cell));
                Draw::fill(&painter, *cell, Corner::Small, <color::BgField as ColorToken>::VALUE);
            }
        }
        for (index, cell) in cells.iter().enumerate() {
            let state = states.get(index).copied().unwrap_or(InteractionState::Rest).inner();
            if index == selected {
                let moved = cell.translate(vec2(slide_left - cell.left(), 0.0));
                Draw::fill(&painter, moved, Corner::Small, StateStyle::Soft.fill(state));
            } else {
                Draw::fill(&painter, *cell, Corner::Small, StateStyle::Clear.fill(state));
            }
        }
        self.paint_dividers(&painter, &cells, states);
        for (index, (cell, segment)) in cells.iter().zip(&self.segments).enumerate() {
            let state = states.get(index).copied().unwrap_or(InteractionState::Rest);
            // Paper: Segmented sheet, row `One disabled`: a segment that cannot be chosen, in a control that can, has
            // its word at the disabled opacity and no fill
            let one_disabled = !disabled && state == InteractionState::Disabled;
            let word = if one_disabled { InteractionState::Disabled.painter(&painter) } else { painter.clone() };
            self.paint_segment(&word, *cell, segment, index == selected, state);
        }
    }

    // the grid look: one button per segment, the selected one in the accent fill
    fn paint_grid(&self, painter: &Painter, rect: Rect, states: &[InteractionState]) {
        let disabled = !states.is_empty() && states.iter().all(|state| *state == InteractionState::Disabled);
        let painter = if disabled { InteractionState::Disabled.painter(painter) } else { painter.clone() };
        for (index, (cell, segment)) in self.cells(rect).iter().zip(&self.segments).enumerate() {
            let state = states.get(index).copied().unwrap_or(InteractionState::Rest);
            let selected = index == *self.selected;
            if state == InteractionState::Focused {
                FocusRing::paint(&painter, *cell, Corner::Control.radius(*cell));
            }
            let style = if selected { StateStyle::Accent } else { StateStyle::Field };
            Draw::fill(&painter, *cell, Corner::Control, style.fill(state.inner()));
            let colour =
                if selected { <color::OnAccent as ColorToken>::VALUE } else { <color::Text as ColorToken>::VALUE };
            match segment {
                Segment::Icon(icon) => icon.paint(&painter, *cell, IconSize::Regular, colour),
                Segment::Text(text) => {
                    let face = if selected { Self::GRID_LABEL_SELECTED } else { Self::GRID_LABEL };
                    let label = UiText::resolve(painter.ctx(), text);
                    let galley = Draw::galley(&painter, &label, face, colour, TextFit::Truncate(cell.width()));
                    painter.galley(cell.center() - galley.size() / 2.0, galley, colour);
                }
            }
        }
    }

    // a divider shows between two neighbours only when neither is selected or touched
    fn paint_dividers(&self, painter: &Painter, cells: &[Rect], states: &[InteractionState]) {
        let quiet = |index: usize| {
            index != *self.selected
                && matches!(states.get(index), Some(InteractionState::Rest | InteractionState::Disabled) | None)
        };
        let stroke = Stroke::new(<spacing::Divider as SpacingToken>::VALUE, <color::LineStrong as ColorToken>::VALUE);
        for (index, cell) in cells.iter().enumerate().skip(1) {
            if quiet(index - 1) && quiet(index) {
                let x = cell.left().round() - stroke.width / 2.0;
                let y = cell.center().y;
                painter.line_segment(
                    [pos2(x, y - Self::DIVIDER_HEIGHT / 2.0), pos2(x, y + Self::DIVIDER_HEIGHT / 2.0)],
                    stroke,
                );
            }
        }
    }

    fn paint_segment(&self, painter: &Painter, cell: Rect, segment: &Segment, selected: bool, state: InteractionState) {
        let color: Color32 = match (selected, state) {
            (true, _) => <color::AccentHover as ColorToken>::VALUE,
            (false, InteractionState::Hover | InteractionState::Pressed | InteractionState::Focused) => {
                <color::Text as ColorToken>::VALUE
            }
            (false, InteractionState::Rest | InteractionState::Disabled) => <color::TextDim as ColorToken>::VALUE,
        };
        match segment {
            Segment::Icon(icon) => icon.paint(painter, cell, IconSize::Regular, color),
            Segment::Text(text) => {
                let face = if selected { FontFace::Semibold } else { FontFace::Regular };
                let label = UiText::resolve(painter.ctx(), text);
                let galley = Draw::galley(
                    painter,
                    &label,
                    TextStyle::new(TypeSize::Note, face),
                    color,
                    TextFit::Truncate(cell.width()),
                );
                painter.galley(cell.center() - galley.size() / 2.0, galley, color);
            }
        }
    }

    // the segment rects inside the inset: icon segments are `--spacing-button` wide, text segments
    // share the width equally
    fn cells(&self, rect: Rect) -> Vec<Rect> {
        if let Some(columns) = self.columns {
            return self.grid_cells(rect, columns);
        }
        let inner = rect.shrink(Self::INSET);
        let count = self.segments.len().max(1);
        let width =
            if self.all_icons() { <spacing::Button as SpacingToken>::VALUE } else { inner.width() / count as f32 };
        (0..self.segments.len())
            .map(|index| {
                Rect::from_min_size(pos2(inner.left() + width * index as f32, inner.top()), vec2(width, inner.height()))
            })
            .collect()
    }

    // the grid: `columns` equal buttons per row, as many rows as the segments need
    fn grid_cells(&self, rect: Rect, columns: usize) -> Vec<Rect> {
        let row = <spacing::Row as SpacingToken>::VALUE;
        let width = (rect.width() - Self::GRID_GAP * (columns as f32 - 1.0)) / columns as f32;
        (0..self.segments.len())
            .map(|index| {
                let (line, column) = (index / columns, index % columns);
                let at = pos2(
                    rect.left() + (width + Self::GRID_GAP) * column as f32,
                    rect.top() + (row + Self::GRID_GAP) * line as f32,
                );
                Rect::from_min_size(at, vec2(width, row))
            })
            .collect()
    }

    fn segment_reason(&self, index: usize) -> Option<&DisabledReason> {
        self.disabled_segments.iter().find(|(at, _)| *at == index).map(|(_, reason)| reason)
    }

    fn all_icons(&self) -> bool {
        !self.segments.is_empty() && self.segments.iter().all(|segment| matches!(segment, Segment::Icon(_)))
    }

    fn track_height(&self) -> f32 {
        let Some(columns) = self.columns else {
            return self.height.unwrap_or(<spacing::Field as SpacingToken>::VALUE);
        };
        let rows = self.segments.len().div_ceil(columns).max(1) as f32;
        let row = <spacing::Row as SpacingToken>::VALUE;
        rows * row + (rows - 1.0) * Self::GRID_GAP
    }
}
