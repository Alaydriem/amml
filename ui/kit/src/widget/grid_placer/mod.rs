mod grid_cell_look;
mod grid_cell_ref;
mod grid_chip_look;
mod grid_pass;
mod grid_piece;
mod grid_placer_response;

pub use grid_cell_look::GridCellLook;
pub use grid_cell_ref::GridCellRef;
pub use grid_chip_look::GridChipLook;
pub use grid_placer_response::GridPlacerResponse;

use amml_i18n::text::{Args, Text};
use egui::epaint::RoundedRect;
use egui::epaint::tessellator::path;
use egui::{
    Color32, Context, CursorIcon, DragAndDrop, EventFilter, Id, Key, LayerId, Order, Painter, Rect, Response, Sense,
    Shape, Stroke, Ui, pos2, vec2,
};
use grid_pass::GridPass;
use grid_piece::GridPiece;

use crate::AmmlUiMessage;
use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::{Icon, IconSize};
use crate::id::AmmlUiId;
use crate::state::{DisabledReason, InteractionState};
use crate::text::UiText;
use crate::tokens::rule::{Elevation, FocusRing};
use crate::tokens::{ColorToken, OpacityToken, SpacingToken, color, opacity, spacing};
use crate::widget::{SectionHeader, Tooltip};

/// Paper: Widgets, `Widget / Grid placer`. A tray of pieces that are not placed, over a grid of
/// `columns` x `rows` cells. The player drags a piece chip onto an empty cell, drags a placed piece
/// to another empty cell, and returns a placed piece to the tray by dropping it on the tray, by
/// letting it go outside the placer, or with the unplace button of a hovered filled cell. A filled
/// cell refuses a drop. Keyboard: Tab focuses a chip, Enter picks it up, the arrow keys move the
/// focus cell, Enter places, Esc lets go, and Delete or Backspace on a focused filled cell
/// unplaces. The placer keeps no model: it reports each change and the caller
/// gives the new cells on the next frame.
pub struct GridPlacer {
    id: Id,
    columns: u16,
    rows: u16,
    pieces: Vec<GridPiece>,
    disabled: Option<DisabledReason>,
}

impl GridPlacer {
    /// The most columns and rows a placer shows; larger values are clamped to it.
    pub const MAX_SIDE: u16 = 32;
    /// Paper: Grid placer sheet, the dash of a dashed cell or ghost border.
    const DASH: f32 = 4.0;
    /// Paper: Grid placer sheet, the gap between the dashes of a dashed border.
    const DASH_GAP: f32 = 3.0;
    const NAME: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Semibold);
    const CAPTION: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);
    const CHIP_NAME: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Regular);

    /// `columns` and `rows` are clamped to 1 ..= `MAX_SIDE`.
    pub fn new(id: Id, columns: u16, rows: u16) -> Self {
        Self {
            id,
            columns: columns.clamp(1, Self::MAX_SIDE),
            rows: rows.clamp(1, Self::MAX_SIDE),
            pieces: Vec::new(),
            disabled: None,
        }
    }

    /// A piece, `key` unique in this placer. `cell` is `None` for a piece in the tray. A piece on a
    /// cell outside the grid, or on a cell an earlier piece holds, waits in the tray.
    pub fn piece(mut self, key: u64, label: Text, cell: Option<GridCellRef>) -> Self {
        self.pieces.push(GridPiece { key, label, cell });
        self
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    /// The columns shown, after the clamp.
    pub fn columns(&self) -> u16 {
        self.columns
    }

    /// The rows shown, after the clamp.
    pub fn rows(&self) -> u16 {
        self.rows
    }

    /// The narrowest a cell gets: 2 x `--spacing-tile-large`. Cells grow equally to fill the width.
    pub fn min_cell_width() -> f32 {
        2.0 * <spacing::TileLarge as SpacingToken>::VALUE
    }

    pub fn cell_height() -> f32 {
        <spacing::TileLarge as SpacingToken>::VALUE
    }

    pub fn chip_height() -> f32 {
        <spacing::Field as SpacingToken>::VALUE
    }

    /// The width a chip takes for `label`: the padding, the grip, the gap and the name.
    pub fn chip_width(ctx: &Context, label: &Text) -> f32 {
        let name = UiText::resolve(ctx, label);
        let pad = <spacing::M as SpacingToken>::VALUE;
        let gap = <spacing::S as SpacingToken>::VALUE;
        pad * 2.0 + IconSize::Small.points() + gap + Draw::measure(ctx, &name, Self::CHIP_NAME).x
    }

    /// Takes the full width, or more when the cells at their narrowest do not fit.
    pub fn show(self, ui: &mut Ui) -> GridPlacerResponse {
        let enabled = self.disabled.is_none();
        let ctx = ui.ctx().clone();
        let (board, tray) = self.sort_pieces();
        let held_id = self.id.with(AmmlUiId::GridHeld);
        let focus_id = self.id.with(AmmlUiId::GridFocusCell);
        let held = if enabled { ui.data(|data| data.get_temp::<u64>(held_id)) } else { None };
        let held = held.filter(|key| tray.iter().any(|&index| self.pieces[index].key == *key));
        let focus = self.clamp(ui.data(|data| data.get_temp::<GridCellRef>(focus_id)).unwrap_or_default());
        let dragged = if enabled { self.dragged(&ctx) } else { None };
        let keyboard = ui.memory(|memory| memory.focused());
        let mut pass = GridPass { held, focus, dragged, keyboard, picked: false, out: GridPlacerResponse::None };

        let gap = <spacing::S as SpacingToken>::VALUE;
        let columns = f32::from(self.columns);
        // in an unbounded ui (a horizontal scroll area) the cells keep their narrowest width
        let narrowest = Self::min_cell_width() * columns + gap * (columns - 1.0);
        let width = Some(ui.available_width()).filter(|width| width.is_finite()).unwrap_or(narrowest);
        let chips = self.chip_places(&ctx, &tray, width);
        let tray_height = chips.iter().map(|chip| chip.bottom()).fold(Self::chip_height(), f32::max);
        let cell_width = ((width - gap * (columns - 1.0)) / columns).max(Self::min_cell_width());
        let grid_size = vec2(
            cell_width * columns + gap * (columns - 1.0),
            Self::cell_height() * f32::from(self.rows) + gap * (f32::from(self.rows) - 1.0),
        );
        let grid_top = SectionHeader::height() + tray_height + <spacing::M as SpacingToken>::VALUE;
        let (rect, whole) =
            ui.allocate_exact_size(vec2(width.max(grid_size.x), grid_top + grid_size.y), Sense::hover());
        let state = if enabled { InteractionState::Rest } else { InteractionState::Disabled };
        let painter = state.painter(ui.painter());

        let header = Rect::from_min_size(rect.min, vec2(width, SectionHeader::height()));
        let count = u64::try_from(tray.len()).unwrap_or(u64::MAX);
        SectionHeader::new(Text::format(AmmlUiMessage::NotPlaced, Args::new().uint("count", count)))
            .paint(&painter, header);
        let tray_rect = Rect::from_min_size(pos2(rect.left(), header.bottom()), vec2(width, tray_height));
        self.show_tray(ui, &painter, (tray_rect, &tray, &chips), &mut pass);

        let grid = Rect::from_min_size(pos2(rect.left(), rect.top() + grid_top), grid_size);
        for row in 0..self.rows {
            for column in 0..self.columns {
                let cell = GridCellRef::new(column, row);
                let min = grid.min
                    + vec2(f32::from(column) * (cell_width + gap), f32::from(row) * (Self::cell_height() + gap));
                let cell_rect = Rect::from_min_size(min, vec2(cell_width, Self::cell_height()));
                let occupant = board[self.index(cell)].map(|index| &self.pieces[index]);
                self.show_cell(ui, &painter, (cell, cell_rect, occupant), &mut pass);
            }
        }
        self.paint_lifted(&ctx, width, pass.dragged);
        self.release_outside(&ctx, rect, &board, &mut pass);

        ui.data_mut(|data| {
            match pass.held {
                Some(key) => {
                    data.insert_temp(held_id, key);
                }
                None => data.remove::<u64>(held_id),
            }
            data.insert_temp(focus_id, pass.focus);
        });
        if let Some(reason) = &self.disabled {
            reason.tooltip().attach(whole);
        }
        pass.out
    }

    /// Paints one cell in `look`, for the gallery and the widget. `number` counts the cells from 1.
    pub fn paint_cell(painter: &Painter, rect: Rect, number: u32, look: &GridCellLook, focused: bool) {
        Self::paint_cell_with(painter, rect, number, look, (focused, false));
    }

    // `paint_cell`, with the unplace button in its hover look when `unplace_hovered`
    fn paint_cell_with(
        painter: &Painter,
        rect: Rect,
        number: u32,
        look: &GridCellLook,
        (focused, unplace_hovered): (bool, bool),
    ) {
        let corner = Corner::Control;
        if focused {
            FocusRing::paint(painter, rect, corner.radius(rect));
        }
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let label = Text::format(AmmlUiMessage::GridCell, Args::new().uint("number", u64::from(number)));
        let pad = <spacing::M as SpacingToken>::VALUE;
        let (main, second) = match look {
            GridCellLook::Empty => {
                Self::dashed(
                    painter,
                    rect,
                    corner,
                    <spacing::Border as SpacingToken>::VALUE,
                    <color::LineStrong as ColorToken>::VALUE,
                );
                (None, (label, faint))
            }
            GridCellLook::DropTarget | GridCellLook::PlaceHere => {
                Draw::fill(painter, rect, corner, <color::AccentSoft as ColorToken>::VALUE);
                let strong = <spacing::BorderStrong as SpacingToken>::VALUE;
                Self::dashed(painter, rect, corner, strong, <color::Accent as ColorToken>::VALUE);
                let message = if matches!(look, GridCellLook::DropTarget) {
                    AmmlUiMessage::DropHere
                } else {
                    AmmlUiMessage::PlaceHere
                };
                (Some((Text::message(message), <color::AccentHover as ColorToken>::VALUE, pad)), (label, faint))
            }
            GridCellLook::Filled(name) => {
                Draw::surface(
                    painter,
                    rect,
                    corner,
                    <color::BgField as ColorToken>::VALUE,
                    Some(<color::LineStrong as ColorToken>::VALUE),
                    Elevation::Flat,
                );
                (Some((name.clone(), <color::Text as ColorToken>::VALUE, pad)), (label, faint))
            }
            GridCellLook::FilledHover(name) => {
                Draw::surface(
                    painter,
                    rect,
                    corner,
                    <color::BgHover as ColorToken>::VALUE,
                    Some(<color::LineStrong as ColorToken>::VALUE),
                    Elevation::Flat,
                );
                Self::paint_unplace(painter, Self::unplace_rect(rect), unplace_hovered);
                let inset = <spacing::ControlSmall as SpacingToken>::VALUE + 2.0 * <spacing::Xs as SpacingToken>::VALUE;
                (Some((name.clone(), <color::Text as ColorToken>::VALUE, inset)), (label, faint))
            }
            GridCellLook::Blocked(name) => {
                Draw::surface(
                    painter,
                    rect,
                    corner,
                    <color::DangerSoft as ColorToken>::VALUE,
                    Some(<color::Danger as ColorToken>::VALUE),
                    Elevation::Flat,
                );
                let danger = <color::Danger as ColorToken>::VALUE;
                (
                    Some((name.clone(), <color::TextDim as ColorToken>::VALUE, pad)),
                    (Text::message(AmmlUiMessage::Taken), danger),
                )
            }
        };
        Self::paint_cell_text(painter, rect, main, second);
    }

    /// Paints one piece chip in `look`, for the gallery and the widget.
    pub fn paint_chip(painter: &Painter, rect: Rect, label: &Text, look: GridChipLook) {
        let corner = Corner::Control;
        if matches!(look, GridChipLook::Focused | GridChipLook::Held) {
            FocusRing::paint(painter, rect, corner.radius(rect));
        }
        let line = <color::LineStrong as ColorToken>::VALUE;
        let accent = <color::Accent as ColorToken>::VALUE;
        let mut content = painter.clone();
        let mut grip = <color::TextFaint as ColorToken>::VALUE;
        match look {
            GridChipLook::Rest | GridChipLook::Focused => {
                Draw::surface(
                    painter,
                    rect,
                    corner,
                    <color::BgField as ColorToken>::VALUE,
                    Some(line),
                    Elevation::Flat,
                );
            }
            GridChipLook::Hover => {
                Draw::surface(
                    painter,
                    rect,
                    corner,
                    <color::BgHover as ColorToken>::VALUE,
                    Some(line),
                    Elevation::Flat,
                );
            }
            GridChipLook::Dragging => {
                Draw::surface(
                    painter,
                    rect,
                    corner,
                    <color::BgActive as ColorToken>::VALUE,
                    Some(accent),
                    Elevation::Floating,
                );
                grip = <color::TextDim as ColorToken>::VALUE;
            }
            GridChipLook::Held => {
                Draw::surface(
                    painter,
                    rect,
                    corner,
                    <color::AccentSoft as ColorToken>::VALUE,
                    Some(accent),
                    Elevation::Flat,
                );
            }
            GridChipLook::Ghost => {
                Self::dashed(painter, rect, corner, <spacing::Border as SpacingToken>::VALUE, line);
                content.multiply_opacity(<opacity::Muted as OpacityToken>::VALUE);
            }
        }
        let pad = <spacing::M as SpacingToken>::VALUE;
        let icon = IconSize::Small.points();
        let grip_rect = Rect::from_min_size(pos2(rect.left() + pad, rect.top()), vec2(icon, rect.height()));
        Icon::Grip.paint(&content, grip_rect, IconSize::Small, grip);
        let left = grip_rect.right() + <spacing::S as SpacingToken>::VALUE;
        let room = (rect.right() - pad - left).max(0.0);
        let colour = <color::Text as ColorToken>::VALUE;
        let name = UiText::resolve(painter.ctx(), label);
        let galley = Draw::galley(&content, &name, Self::CHIP_NAME, colour, TextFit::Truncate(room));
        content.galley(pos2(left, rect.center().y - galley.size().y / 2.0), galley, colour);
    }

    // the tray's chips, and the tray as a drop zone for a placed piece
    fn show_tray(
        &self,
        ui: &Ui,
        painter: &Painter,
        (rect, tray, chips): (Rect, &[usize], &[Rect]),
        pass: &mut GridPass,
    ) {
        let enabled = self.disabled.is_none();
        let zone = ui.interact(rect, self.id.with(AmmlUiId::GridTray), Sense::hover());
        if let Some(key) = self.payload_over(&zone)
            && !tray.iter().any(|&index| self.pieces[index].key == key)
            && Self::released(&zone)
        {
            pass.out = GridPlacerResponse::Unplaced { key };
        }
        for (&index, place) in tray.iter().zip(chips) {
            let piece = &self.pieces[index];
            let chip = place.translate(rect.min.to_vec2());
            let sense = if enabled { Sense::click_and_drag() } else { Sense::hover() };
            let response = ui.interact(chip, self.chip_id(piece.key), sense);
            if enabled {
                response.dnd_set_drag_payload((self.id, piece.key));
                if response.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::Grab);
                }
            }
            if response.clicked() {
                pass.held = if pass.held == Some(piece.key) { None } else { Some(piece.key) };
                pass.picked = true;
                if pass.held.is_some() {
                    ui.memory_mut(|memory| memory.request_focus(self.cell_id(pass.focus)));
                }
            }
            let look = if pass.dragged == Some(piece.key) {
                GridChipLook::Ghost
            } else if pass.held == Some(piece.key) {
                GridChipLook::Held
            } else {
                match InteractionState::of(&response, enabled) {
                    InteractionState::Hover | InteractionState::Pressed => GridChipLook::Hover,
                    InteractionState::Focused => GridChipLook::Focused,
                    InteractionState::Rest | InteractionState::Disabled => GridChipLook::Rest,
                }
            };
            Self::paint_chip(painter, chip, &piece.label, look);
        }
    }

    // one cell: its drop, its keys, its unplace button and its look
    fn show_cell(
        &self,
        ui: &Ui,
        painter: &Painter,
        (cell, rect, occupant): (GridCellRef, Rect, Option<&GridPiece>),
        pass: &mut GridPass,
    ) {
        let enabled = self.disabled.is_none();
        let mut sense = Sense::hover();
        if enabled {
            if occupant.is_some() {
                // a click too, so a press on the unplace button does not start a drag at once
                sense |= Sense::CLICK | Sense::DRAG;
            } else if pass.held.is_some() {
                sense |= Sense::CLICK;
            }
            if cell == pass.focus {
                sense |= Sense::FOCUSABLE;
            }
        }
        let response = ui.interact(rect, self.cell_id(cell), sense);
        let key = occupant.map(|piece| piece.key);
        if let Some(key) = key
            && enabled
        {
            response.dnd_set_drag_payload((self.id, key));
        }
        let mut look = match occupant {
            None => GridCellLook::Empty,
            Some(piece) => GridCellLook::Filled(piece.label.clone()),
        };
        let over = self.payload_over(&response).filter(|over| Some(*over) != key);
        if let Some(over) = over {
            look = match occupant {
                None => {
                    if Self::released(&response) {
                        pass.out = GridPlacerResponse::Placed { key: over, cell };
                    }
                    GridCellLook::DropTarget
                }
                Some(piece) => GridCellLook::Blocked(piece.label.clone()),
            };
        } else if key.is_some() && pass.dragged == key && !response.contains_pointer() {
            // the piece is lifted off this cell
            look = GridCellLook::Empty;
        }
        if response.has_focus() && pass.keyboard == Some(response.id) {
            self.cell_keys(ui, cell, key, pass);
        }
        if response.clicked()
            && !pass.picked
            && key.is_none()
            && let Some(held) = pass.held.take()
        {
            pass.out = GridPlacerResponse::Placed { key: held, cell };
        }
        if let Some(piece) = occupant
            && enabled
            && pass.dragged.is_none()
            && response.contains_pointer()
        {
            look = GridCellLook::FilledHover(piece.label.clone());
            let button = Self::unplace_rect(rect);
            let unplace = ui.interact(button, self.id.with(AmmlUiId::GridUnplace).with(piece.key), Sense::CLICK);
            if unplace.clicked() {
                pass.out = GridPlacerResponse::Unplaced { key: piece.key };
            }
            let hovered = unplace.hovered();
            Tooltip::new(Text::message(AmmlUiMessage::Unplace))
                .attach(unplace.on_hover_cursor(CursorIcon::PointingHand));
            Self::paint_cell_with(painter, rect, self.number(cell), &look, (false, hovered));
            return;
        }
        if response.has_focus() && pass.held.is_some() && key.is_none() && over.is_none() {
            look = GridCellLook::PlaceHere;
        }
        let focused = InteractionState::of(&response, enabled) == InteractionState::Focused;
        Self::paint_cell(painter, rect, self.number(cell), &look, focused);
    }

    // the keyboard route on the focus cell
    fn cell_keys(&self, ui: &Ui, cell: GridCellRef, key: Option<u64>, pass: &mut GridPass) {
        let filter = EventFilter {
            horizontal_arrows: true,
            vertical_arrows: true,
            escape: pass.held.is_some(),
            ..EventFilter::default()
        };
        let id = self.cell_id(cell);
        ui.memory_mut(|memory| memory.set_focus_lock_filter(id, filter));
        let (step_x, step_y, enter, delete, escape) = ui.input(|input| {
            (
                i32::from(input.key_pressed(Key::ArrowRight)) - i32::from(input.key_pressed(Key::ArrowLeft)),
                i32::from(input.key_pressed(Key::ArrowDown)) - i32::from(input.key_pressed(Key::ArrowUp)),
                input.key_pressed(Key::Enter),
                input.key_pressed(Key::Delete) || input.key_pressed(Key::Backspace),
                input.key_pressed(Key::Escape),
            )
        });
        let next = self.clamp_step(cell, step_x, step_y);
        if next != cell {
            pass.focus = next;
            ui.memory_mut(|memory| memory.request_focus(self.cell_id(next)));
        }
        if enter
            && !pass.picked
            && key.is_none()
            && let Some(held) = pass.held.take()
        {
            pass.out = GridPlacerResponse::Placed { key: held, cell };
        }
        if delete && let Some(key) = key {
            pass.out = GridPlacerResponse::Unplaced { key };
        }
        if escape && let Some(held) = pass.held.take() {
            ui.memory_mut(|memory| memory.request_focus(self.chip_id(held)));
        }
    }

    // the chip under the pointer while one of this placer's pieces is dragged
    fn paint_lifted(&self, ctx: &Context, width: f32, dragged: Option<u64>) {
        let Some(piece) = dragged.and_then(|key| self.pieces.iter().find(|piece| piece.key == key)) else {
            return;
        };
        let Some(pointer) = ctx.pointer_interact_pos() else {
            return;
        };
        let painter = ctx.layer_painter(LayerId::new(Order::Tooltip, self.id.with(AmmlUiId::GridDrag)));
        let size = vec2(Self::chip_width(ctx, &piece.label).min(width), Self::chip_height());
        Self::paint_chip(&painter, Rect::from_center_size(pointer, size), &piece.label, GridChipLook::Dragging);
    }

    // a placed piece let go outside the whole placer goes back to the tray; over the placer, the
    // cell or the tray under the pointer decided
    fn release_outside(&self, ctx: &Context, rect: Rect, board: &[Option<usize>], pass: &mut GridPass) {
        let Some(key) = pass.dragged else {
            return;
        };
        let placed = board.iter().flatten().any(|&index| self.pieces[index].key == key);
        let (released, pointer) = ctx.input(|input| (input.pointer.any_released(), input.pointer.latest_pos()));
        let outside = pointer.is_none_or(|pointer| !rect.contains(pointer));
        if placed && released && outside && pass.out == GridPlacerResponse::None && self.dragged(ctx).is_some() {
            DragAndDrop::clear_payload(ctx);
            pass.out = GridPlacerResponse::Unplaced { key };
        }
    }

    // which piece is on each cell, row by row, and the pieces in the tray
    fn sort_pieces(&self) -> (Vec<Option<usize>>, Vec<usize>) {
        let mut board = vec![None; usize::from(self.columns) * usize::from(self.rows)];
        let mut tray = Vec::new();
        for (index, piece) in self.pieces.iter().enumerate() {
            let slot = piece
                .cell
                .filter(|cell| cell.column < self.columns && cell.row < self.rows)
                .map(|cell| self.index(cell))
                .filter(|&slot| board[slot].is_none());
            match slot {
                Some(slot) => board[slot] = Some(index),
                None => tray.push(index),
            }
        }
        (board, tray)
    }

    // the chips' rects from the tray's top left, flowing into rows `width` wide
    fn chip_places(&self, ctx: &Context, tray: &[usize], width: f32) -> Vec<Rect> {
        let gap = <spacing::S as SpacingToken>::VALUE;
        let height = Self::chip_height();
        let mut places = Vec::with_capacity(tray.len());
        let (mut x, mut y) = (0.0, 0.0);
        for &index in tray {
            let chip = Self::chip_width(ctx, &self.pieces[index].label).min(width);
            if x > 0.0 && x + chip > width {
                x = 0.0;
                y += height + gap;
            }
            places.push(Rect::from_min_size(pos2(x, y), vec2(chip, height)));
            x += chip + gap;
        }
        places
    }

    // this placer's piece being dragged, from egui's drag-and-drop payload
    fn dragged(&self, ctx: &Context) -> Option<u64> {
        DragAndDrop::payload::<(Id, u64)>(ctx).filter(|payload| payload.0 == self.id).map(|payload| payload.1)
    }

    // this placer's piece held over `response`
    fn payload_over(&self, response: &Response) -> Option<u64> {
        if self.disabled.is_some() {
            return None;
        }
        response.dnd_hover_payload::<(Id, u64)>().filter(|payload| payload.0 == self.id).map(|payload| payload.1)
    }

    // the pointer let go of the payload over `response` this frame; the payload is taken
    fn released(response: &Response) -> bool {
        response.dnd_release_payload::<(Id, u64)>().is_some()
    }

    fn clamp(&self, cell: GridCellRef) -> GridCellRef {
        GridCellRef::new(cell.column.min(self.columns - 1), cell.row.min(self.rows - 1))
    }

    fn clamp_step(&self, cell: GridCellRef, step_x: i32, step_y: i32) -> GridCellRef {
        let column = (i32::from(cell.column) + step_x).clamp(0, i32::from(self.columns) - 1);
        let row = (i32::from(cell.row) + step_y).clamp(0, i32::from(self.rows) - 1);
        GridCellRef::new(u16::try_from(column).unwrap_or(cell.column), u16::try_from(row).unwrap_or(cell.row))
    }

    fn index(&self, cell: GridCellRef) -> usize {
        usize::from(cell.row) * usize::from(self.columns) + usize::from(cell.column)
    }

    fn number(&self, cell: GridCellRef) -> u32 {
        u32::from(cell.row) * u32::from(self.columns) + u32::from(cell.column) + 1
    }

    fn chip_id(&self, key: u64) -> Id {
        self.id.with(AmmlUiId::GridChip).with(key)
    }

    fn cell_id(&self, cell: GridCellRef) -> Id {
        self.id.with(AmmlUiId::GridCell).with(cell)
    }

    // the unplace button: `--spacing-control-small` square, `--spacing-xs` in from the top right
    fn unplace_rect(cell: Rect) -> Rect {
        let side = <spacing::ControlSmall as SpacingToken>::VALUE;
        let inset = <spacing::Xs as SpacingToken>::VALUE;
        Rect::from_min_size(pos2(cell.right() - inset - side, cell.top() + inset), vec2(side, side))
    }

    fn paint_unplace(painter: &Painter, rect: Rect, hovered: bool) {
        Draw::fill(painter, rect, Corner::Small, <color::BgActive as ColorToken>::VALUE);
        let colour = if hovered { <color::Text as ColorToken>::VALUE } else { <color::TextDim as ColorToken>::VALUE };
        Icon::Cancel.paint(painter, rect, IconSize::Small, colour);
    }

    // the main line (text, colour, side padding) over the second line, centred in the cell
    fn paint_cell_text(painter: &Painter, rect: Rect, main: Option<(Text, Color32, f32)>, second: (Text, Color32)) {
        let main_height = if main.is_some() { TypeSize::Label.line_height() } else { 0.0 };
        let mut top = rect.center().y - (main_height + TypeSize::Caption.line_height()) / 2.0;
        let pad = <spacing::M as SpacingToken>::VALUE;
        if let Some((text, colour, inset)) = main {
            let text = UiText::resolve(painter.ctx(), &text);
            let room = (rect.width() - 2.0 * inset).max(0.0);
            let galley = Draw::galley(painter, &text, Self::NAME, colour, TextFit::Truncate(room));
            painter.galley(pos2(rect.center().x - galley.size().x / 2.0, top), galley, colour);
            top += main_height;
        }
        let (text, colour) = second;
        let text = UiText::resolve(painter.ctx(), &text);
        let room = (rect.width() - 2.0 * pad).max(0.0);
        let galley = Draw::galley(painter, &text, Self::CAPTION, colour, TextFit::Truncate(room));
        painter.galley(pos2(rect.center().x - galley.size().x / 2.0, top), galley, colour);
    }

    // a dashed outline `width` wide inside `rect`
    fn dashed(painter: &Painter, rect: Rect, corner: Corner, width: f32, colour: Color32) {
        let mut points = Vec::new();
        path::rounded_rectangle(&mut points, RoundedRect::new(rect.shrink(width / 2.0), corner.radius(rect)));
        if let Some(first) = points.first().copied() {
            points.push(first);
        }
        painter.extend(Shape::dashed_line(&points, Stroke::new(width, colour), Self::DASH, Self::DASH_GAP));
    }
}
