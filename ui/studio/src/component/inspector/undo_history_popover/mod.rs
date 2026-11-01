mod undo_history_action;
mod undo_history_view;

pub use undo_history_action::UndoHistoryAction;
pub use undo_history_view::UndoHistoryView;

use amml_i18n::text::{Args, Text, UserText};
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, color};
use egui::{Align2, CursorIcon, Id, Rect, ScrollArea, Sense, Ui, UiBuilder, pos2, vec2};

use crate::id::StudioUiId;
use crate::message::StudioUiMessage;

/// Paper: Menus + Dialogs, `Undo history` (`46V-0`). The popover under the top bar's history button: its title,
/// then one row per step, numbered from 1. With no steps it says there is nothing to undo.
pub struct UndoHistoryPopover<'a> {
    view: &'a UndoHistoryView,
    max_rows: usize,
}

impl<'a> UndoHistoryPopover<'a> {
    /// Paper: the popover is 240 wide with 6 inside.
    pub const WIDTH: f32 = 240.0;
    const PADDING: f32 = 6.0;
    // Paper: the title has 4 above, 6 below and 8 each side; rows are 26 high, 2 apart, 8 each side, 4 under the line
    const TITLE_TOP: f32 = 4.0;
    const TITLE_BOTTOM: f32 = 6.0;
    const SIDE: f32 = 8.0;
    const ROW: f32 = 26.0;
    const ROW_GAP: f32 = 2.0;
    const LIST_TOP: f32 = 4.0;
    const EMPTY: f32 = 30.0;
    /// Paper draws six steps; past this many the list scrolls (not in Paper).
    pub const MAX_ROWS: usize = 12;
    const TITLE: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);
    const STEP: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);

    pub fn new(view: &'a UndoHistoryView) -> Self {
        Self { view, max_rows: Self::MAX_ROWS }
    }

    /// The most rows the list shows before it scrolls.
    pub fn max_rows(mut self, max_rows: usize) -> Self {
        self.max_rows = max_rows.max(1);
        self
    }

    // the height of `count` rows
    fn rows(count: usize) -> f32 {
        Self::ROW * count as f32 + Self::ROW_GAP * count.saturating_sub(1) as f32
    }

    pub fn show(&self, ui: &mut Ui) -> Option<UndoHistoryAction> {
        let title_height = Self::TITLE_TOP + TypeSize::Help.line_height() + Self::TITLE_BOTTOM;
        let count = self.view.steps.len();
        let list = if count == 0 { Self::EMPTY } else { Self::LIST_TOP + Self::rows(count.min(self.max_rows)) };
        let size = vec2(Self::WIDTH, Self::PADDING * 2.0 + title_height + 1.0 + list);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let raised = <color::BgRaised as ColorToken>::VALUE;
        let strong = <color::LineStrong as ColorToken>::VALUE;
        Draw::surface(ui.painter(), rect, Corner::Control, raised, Some(strong), Elevation::Floating);
        let inner = rect.shrink(Self::PADDING);
        let ink = <color::Text as ColorToken>::VALUE;
        let title = Text::message(StudioUiMessage::UndoHistory);
        let at = pos2(inner.left() + Self::SIDE, inner.top() + Self::TITLE_TOP);
        Draw::text(ui.painter(), at, Align2::LEFT_TOP, &title, Self::TITLE, ink, TextFit::Line);
        let line = inner.top() + title_height;
        Draw::divider(ui.painter(), inner.x_range(), line, strong);
        if count == 0 {
            let empty = Text::message(StudioUiMessage::NothingToUndo);
            let dim = <color::TextDim as ColorToken>::VALUE;
            let at = pos2(inner.left() + Self::SIDE, line + 1.0 + Self::EMPTY / 2.0);
            Draw::text(ui.painter(), at, Align2::LEFT_CENTER, &empty, Self::STEP, dim, TextFit::Line);
            return None;
        }
        let view = Rect::from_min_max(pos2(inner.left(), line + 1.0 + Self::LIST_TOP), inner.max);
        let mut child = ui.new_child(UiBuilder::new().max_rect(view));
        // the newest step is last; the list opens at the bottom so it shows
        let scroll = ScrollArea::vertical()
            .id_salt(Id::unique(StudioUiId::UndoHistory))
            .auto_shrink([false, false])
            .max_height(view.height())
            .stick_to_bottom(true);
        let picked = scroll.show(&mut child, |ui| {
            let (area, _) = ui.allocate_exact_size(vec2(view.width(), Self::rows(count)), Sense::hover());
            let mut picked = None;
            for (index, step) in self.view.steps.iter().enumerate() {
                let top = area.top() + (Self::ROW + Self::ROW_GAP) * index as f32;
                let row = Rect::from_min_size(pos2(area.left(), top), vec2(area.width(), Self::ROW));
                if !ui.is_rect_visible(row) {
                    continue;
                }
                let response = ui.allocate_rect(row, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
                if response.hovered() {
                    Draw::fill(ui.painter(), row, Corner::Small, <color::BgHover as ColorToken>::VALUE);
                }
                let name = UserText::from_model(UiText::resolve(ui.ctx(), step));
                let args = Args::new().uint("index", index as u64 + 1).user("step", name);
                let label = Text::format(StudioUiMessage::UndoStep, args);
                let room = (row.width() - Self::SIDE * 2.0).max(0.0);
                let at = pos2(row.left() + Self::SIDE, row.center().y);
                Draw::text(ui.painter(), at, Align2::LEFT_CENTER, &label, Self::STEP, ink, TextFit::Truncate(room));
                if response.clicked() {
                    picked = Some(UndoHistoryAction::JumpTo(index));
                }
            }
            picked
        });
        let picked = picked.inner;
        // `allocate_rect` moved the cursor to each row; the page goes on under the popover
        ui.advance_cursor_after_rect(rect);
        picked
    }
}
