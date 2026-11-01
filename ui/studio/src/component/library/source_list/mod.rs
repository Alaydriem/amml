mod source_list_action;
mod source_list_view;
mod source_row;
mod source_row_kind;

pub use source_list_action::SourceListAction;
pub use source_list_view::SourceListView;
pub use source_row::SourceRow;
pub use source_row_kind::SourceRowKind;

use amml_ui::draw::{Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::state::InteractionState;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{ButtonKind, Leading, TextButton};
use egui::{CursorIcon, Id, Rect, Sense, Ui, pos2, vec2};

use crate::id::StudioUiId;

/// Paper: Screen / Library (`DX-0`), the Sources block under the replay list. A tracked title, a row for each source
/// (a folder icon, the name, and a small tag or a remove button) and the Add folder button. It takes the width left
/// and is `height` high.
pub struct SourceList<'a> {
    view: &'a SourceListView,
}

impl<'a> SourceList<'a> {
    /// Paper: DX-0, the room above the title and the gap between the parts of the block.
    const TOP: f32 = 2.0;
    const GAP: f32 = 6.0;
    /// Paper: DX-0, the row height, its room left of the icon, the icon cell width and the gap after it.
    const ROW: f32 = 24.0;
    const ROW_LEFT: f32 = 2.0;
    const ICON_CELL: f32 = 14.0;
    const ROW_GAP: f32 = 10.0;
    /// Paper: DX-0, the width of the remove cell at the right end of a folder row.
    const REMOVE: f32 = 24.0;
    /// Paper: DX-0, the room above the button, and the button size.
    const BUTTON_TOP: f32 = 4.0;
    const BUTTON: egui::Vec2 = vec2(140.0, 30.0);
    const TITLE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();
    const NAME: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);
    const TAG: TextStyle = TextStyle::new(TypeSize::Micro, FontFace::Semibold);

    pub fn new(view: &'a SourceListView) -> Self {
        Self { view }
    }

    /// The block's height for `count` rows.
    pub fn height(count: usize) -> f32 {
        Self::TOP
            + TypeSize::Caption.line_height()
            + Self::GAP
            + count as f32 * (Self::ROW + Self::GAP)
            + Self::BUTTON_TOP
            + Self::BUTTON.y
    }

    /// Shows the block; returns the action of the control that was clicked.
    pub fn show(&self, ui: &mut Ui) -> Option<SourceListAction> {
        let size = vec2(ui.available_width(), Self::height(self.view.sources.len()));
        let (rect, frame) = ui.allocate_exact_size(size, Sense::hover());
        let painter = ui.painter().clone();
        let title = UiText::resolve(ui.ctx(), &self.view.title);
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let title = Draw::galley(&painter, &title, Self::TITLE, faint, TextFit::Truncate(rect.width()));
        let mut y = rect.top() + Self::TOP;
        let title_height = TypeSize::Caption.line_height();
        painter.galley(pos2(rect.left(), y + (title_height - title.size().y) / 2.0), title, faint);
        y += title_height + Self::GAP;
        let mut action = None;
        for (index, source) in self.view.sources.iter().enumerate() {
            let row = Rect::from_min_size(pos2(rect.left(), y), vec2(rect.width(), Self::ROW));
            if let Some(clicked) = self.paint_row(ui, row, index, source, Self::id(frame.id, index)) {
                action = Some(clicked);
            }
            y += Self::ROW + Self::GAP;
        }
        let button_rect = Rect::from_min_size(pos2(rect.left(), y + Self::BUTTON_TOP), Self::BUTTON);
        let button = TextButton::new(ButtonKind::Outline, self.view.add.clone()).leading(Leading::Icon(Icon::Add));
        let response = ui.interact(button_rect, Self::id(frame.id, self.view.sources.len()), Sense::click());
        button.paint(&painter, button_rect, InteractionState::of(&response, true));
        if response.on_hover_cursor(CursorIcon::PointingHand).clicked() {
            action = Some(SourceListAction::Add);
        }
        action
    }

    // one row: the icon, the name and the tag or the remove cell; returns the remove action when it was clicked
    fn paint_row(&self, ui: &Ui, row: Rect, index: usize, source: &SourceRow, id: Id) -> Option<SourceListAction> {
        let painter = ui.painter();
        let centre = row.center().y;
        let icon = IconSize::Small.points();
        let cell = Rect::from_min_size(pos2(row.left() + Self::ROW_LEFT, row.top()), vec2(Self::ICON_CELL, Self::ROW));
        let slot = Rect::from_center_size(cell.center(), vec2(icon, icon));
        Icon::Folder.paint(painter, slot, IconSize::Small, <color::TextFaint as ColorToken>::VALUE);
        let left = cell.right() + Self::ROW_GAP;
        let dim = <color::TextDim as ColorToken>::VALUE;
        let removable = matches!(source.kind, SourceRowKind::Folder);
        let right = if removable { row.right() - Self::REMOVE - Self::ROW_GAP } else { row.right() };
        let name = UiText::resolve(ui.ctx(), &source.name);
        let galley = Draw::galley(painter, &name, Self::NAME, dim, TextFit::Truncate((right - left).max(0.0)));
        let name_right = left + galley.size().x;
        painter.galley(pos2(left, centre - galley.size().y / 2.0), galley, dim);
        match &source.kind {
            SourceRowKind::Automatic { tag } => {
                let ink = <color::AccentHover as ColorToken>::VALUE;
                let tag = UiText::resolve(ui.ctx(), tag);
                let galley = Draw::galley(painter, &tag, Self::TAG, ink, TextFit::Line);
                painter.galley(pos2(name_right + Self::ROW_GAP, centre - galley.size().y / 2.0), galley, ink);
                None
            }
            SourceRowKind::Folder => {
                let remove =
                    Rect::from_min_size(pos2(row.right() - Self::REMOVE, row.top()), vec2(Self::REMOVE, Self::ROW));
                let response = ui.interact(remove, id, Sense::click());
                let ink = if response.hovered() { <color::Text as ColorToken>::VALUE } else { dim };
                Icon::Close.paint(
                    painter,
                    Rect::from_center_size(remove.center(), vec2(icon, icon)),
                    IconSize::Small,
                    ink,
                );
                response.on_hover_cursor(CursorIcon::PointingHand).clicked().then_some(SourceListAction::Remove(index))
            }
        }
    }

    // each control of the block has its own id, so two lists on one screen do not share them
    fn id(base: Id, index: usize) -> Id {
        base.with(StudioUiId::SourceList).with(index)
    }
}
