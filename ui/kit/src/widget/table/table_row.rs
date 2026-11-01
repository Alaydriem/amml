use amml_i18n::text::Text;
use egui::{Align2, CursorIcon, Painter, Pos2, Rangef, Rect, Response, Sense, Ui, UiBuilder, pos2, vec2};

use super::{TableAlign, TableColumn, TableLayout, TableRowKind, TableRowSize};
use crate::AmmlUiMessage;
use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::{Icon, IconSize};
use crate::state::{InteractionState, StateStyle};
use crate::text::UiText;
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, OpacityToken, SpacingToken, color, opacity, spacing};

/// Paper: Mod manager, `Widget / Table header + Table row`, a row. The caller draws each cell; the
/// row draws its box, its line, the capture prompt and the reason line. The whole row is one click
/// target. It takes the full width.
pub struct TableRow<'c> {
    columns: &'c [TableColumn],
    kind: TableRowKind,
    size: TableRowSize,
    aligns: &'c [TableAlign],
}

impl<'c> TableRow<'c> {
    /// Paper: Table, the gap between a line and the one under it in a cell.
    const LINE_GAP: f32 = 2.0;
    const HELP: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);
    const PROMPT: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);
    /// The chord column, which the capture prompt takes.
    const CHORD: usize = 1;

    pub fn new(columns: &'c [TableColumn], kind: TableRowKind) -> Self {
        Self { columns, kind, size: TableRowSize::Regular, aligns: &[] }
    }

    /// The row's size; the default is `TableRowSize::Regular`.
    pub fn size(mut self, size: TableRowSize) -> Self {
        self.size = size;
        self
    }

    /// Each column's alignment, in column order; a column without one starts at the start edge. A
    /// cell's widgets are laid out toward its aligned edge.
    pub fn aligns(mut self, aligns: &'c [TableAlign]) -> Self {
        self.aligns = aligns;
        self
    }

    /// Shows the row; `cells` draws column `index` into the `Ui` it is given. While capturing, the
    /// chord column is the prompt and `cells` is not asked for it.
    pub fn show(self, ui: &mut Ui, mut cells: impl FnMut(usize, &mut Ui)) -> Response {
        let margin = if self.kind.boxed() { <spacing::Xs as SpacingToken>::VALUE } else { 0.0 };
        let content = self.content_height();
        let height = (content + 2.0 * self.size.pad_y()).max(self.size.min_height()) + 2.0 * margin;
        let disabled = matches!(self.kind, TableRowKind::Disabled(_));
        let sense = if disabled { Sense::hover() } else { Sense::click() };
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), height), sense);
        let state = InteractionState::of(&response, !disabled);
        let painter = state.painter(ui.painter());
        let body = rect.shrink2(vec2(0.0, margin));
        self.paint_body(&painter, body, state);

        let cell_rects = TableLayout::cells(self.columns, body);
        let top = body.center().y - content / 2.0;
        for (index, cell) in cell_rects.iter().enumerate() {
            if index == Self::CHORD && matches!(self.kind, TableRowKind::Capturing) {
                Self::paint_prompt(&painter, cell.left(), body.center().y);
                continue;
            }
            // the action cell sits on the first line when a reason goes under it
            let place = match (&self.kind, index) {
                (TableRowKind::Conflict(_) | TableRowKind::Disabled(_), 0) => {
                    Rect::from_min_size(pos2(cell.left(), top), vec2(cell.width(), TypeSize::Body.line_height()))
                }
                _ => *cell,
            };
            let align = self.aligns.get(index).copied().unwrap_or_default();
            let mut child = ui.new_child(UiBuilder::new().max_rect(place).layout(align.layout()));
            child.style_mut().interaction.selectable_labels = false;
            if disabled {
                // as `SettingRow` does: a muted row's cell widgets take no clicks either. egui's own
                // disabled fade is off, so the row is faded once, by `opacity::Muted`
                child.style_mut().visuals.disabled_alpha = 1.0;
                child.disable();
                child.multiply_opacity(<opacity::Muted as OpacityToken>::VALUE);
            }
            cells(index, &mut child);
        }
        if let Some(first) = cell_rects.first() {
            let y = top + TypeSize::Body.line_height() + Self::LINE_GAP;
            self.paint_reason(&painter, pos2(first.left(), y), first.width());
        }
        if disabled { response } else { response.on_hover_cursor(CursorIcon::PointingHand) }
    }

    // the tallest part the row draws itself: a reason under the action, or the two prompt lines
    fn content_height(&self) -> f32 {
        let help = TypeSize::Help.line_height();
        match self.kind {
            TableRowKind::Plain => TypeSize::Body.line_height(),
            TableRowKind::Capturing => 2.0 * help + Self::LINE_GAP,
            TableRowKind::Conflict(_) | TableRowKind::Disabled(_) => {
                TypeSize::Body.line_height() + Self::LINE_GAP + help
            }
        }
    }

    // the box of a capturing or conflict row; the hover fill and the line under other rows
    fn paint_body(&self, painter: &Painter, body: Rect, state: InteractionState) {
        let boxed = match self.kind {
            TableRowKind::Capturing => {
                Some((<color::AccentSoft as ColorToken>::VALUE, <color::Accent as ColorToken>::VALUE))
            }
            TableRowKind::Conflict(_) => {
                Some((<color::DangerSoft as ColorToken>::VALUE, <color::Danger as ColorToken>::VALUE))
            }
            TableRowKind::Plain | TableRowKind::Disabled(_) => None,
        };
        if let Some((fill, border)) = boxed {
            Draw::fill(painter, body, Corner::Control, fill);
            Draw::stroke(painter, body, Corner::Control, border);
            return;
        }
        if state == InteractionState::Focused {
            FocusRing::paint(painter, body, Corner::Control.radius(body));
        }
        Draw::fill(painter, body, Corner::Control, StateStyle::Clear.fill(state.inner()));
        let y = body.bottom() - <spacing::Divider as SpacingToken>::VALUE;
        Draw::divider(painter, Rangef::new(body.left(), body.right()), y, <color::Line as ColorToken>::VALUE);
    }

    // "Press a key" over "Esc cancels", from `left`, centred on `centre`
    fn paint_prompt(painter: &Painter, left: f32, centre: f32) {
        let help = TypeSize::Help.line_height();
        let top = centre - help - Self::LINE_GAP / 2.0;
        let accent = <color::Accent as ColorToken>::VALUE;
        let dim = <color::TextDim as ColorToken>::VALUE;
        let press = Text::message(AmmlUiMessage::PressAKey);
        let esc = Text::message(AmmlUiMessage::EscCancels);
        Draw::text(painter, pos2(left, top), Align2::LEFT_TOP, &press, Self::PROMPT, accent, TextFit::Line);
        let below = pos2(left, top + help + Self::LINE_GAP);
        Draw::text(painter, below, Align2::LEFT_TOP, &esc, Self::HELP, dim, TextFit::Line);
    }

    // the reason under the action: a danger line with an icon, or a dim line
    fn paint_reason(&self, painter: &Painter, at: Pos2, width: f32) {
        let (reason, colour, icon) = match &self.kind {
            TableRowKind::Conflict(reason) => (reason, <color::Danger as ColorToken>::VALUE, true),
            TableRowKind::Disabled(reason) => (reason, <color::TextDim as ColorToken>::VALUE, false),
            TableRowKind::Plain | TableRowKind::Capturing => return,
        };
        let mut x = at.x;
        if icon {
            let side = IconSize::Small.points();
            let slot =
                Rect::from_min_size(pos2(x, at.y + (TypeSize::Help.line_height() - side) / 2.0), vec2(side, side));
            Icon::Warning.paint(painter, slot, IconSize::Small, colour);
            x += side + <spacing::Xs as SpacingToken>::VALUE;
        }
        let text = UiText::resolve(painter.ctx(), reason);
        let galley = Draw::galley(painter, &text, Self::HELP, colour, TextFit::Truncate((at.x + width - x).max(0.0)));
        painter.galley(pos2(x, at.y), galley, colour);
    }
}
