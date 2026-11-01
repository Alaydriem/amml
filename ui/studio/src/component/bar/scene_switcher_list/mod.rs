mod scene_switcher_footer;
mod scene_switcher_list_action;
mod scene_switcher_list_view;
mod scene_switcher_original;
mod scene_switcher_row;

pub use scene_switcher_footer::SceneSwitcherFooter;
pub use scene_switcher_list_action::SceneSwitcherListAction;
pub use scene_switcher_list_view::SceneSwitcherListView;
pub use scene_switcher_original::SceneSwitcherOriginal;
pub use scene_switcher_row::SceneSwitcherRow;

use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::state::{InteractionState, StateStyle};
use amml_ui::text::UiText;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, ContainerToken, SpacingToken, color, spacing};
use egui::{Color32, CursorIcon, Id, Pos2, Rect, Sense, Ui, pos2, vec2};

use crate::id::StudioUiId;
use crate::tokens::container;

/// Paper: Editor components, `Editor / Scene switcher list` (`GFV-0`). The open scene switcher: a head with a hint,
/// the Original row, one row per scene (colour dot, name, range, camera count and a status pill) and the actions
/// under a line. It is `--container-scene-menu` wide.
pub struct SceneSwitcherList<'a> {
    view: &'a SceneSwitcherListView,
}

impl<'a> SceneSwitcherList<'a> {
    /// Paper: Scene switcher list sheet, the room left and right of the rows.
    const PADDING: f32 = 6.0;
    /// Paper: Scene switcher list sheet, the room under the last action.
    const PADDING_BOTTOM: f32 = 10.0;
    /// Paper: Scene switcher list sheet, the head height and its room left and right.
    const HEADER: f32 = 36.0;
    const HEADER_PADDING: f32 = 10.0;
    /// Paper: Scene switcher list sheet, the row height, the gap between rows, and the room left and right.
    const ROW: f32 = 38.0;
    const ROW_GAP: f32 = 2.0;
    const ROW_PADDING_LEFT: f32 = 12.0;
    const ROW_PADDING_RIGHT: f32 = 10.0;
    /// Paper: Scene switcher list sheet, the gap between the parts of a row.
    const GAP: f32 = 12.0;
    /// Paper: Scene switcher list sheet, the colour dot, the name column and the status column.
    const DOT: f32 = 9.0;
    const NAME_COLUMN: f32 = 140.0;
    const STATUS_COLUMN: f32 = 96.0;
    /// Paper: Scene switcher list sheet, the gap between the name and its check.
    const CHECK_GAP: f32 = 10.0;
    /// Paper: Scene switcher list sheet, the status pill: height, room left and right, gap after its dot, dot size,
    /// and how much of the scene colour its fill and its line show.
    const PILL: f32 = 20.0;
    const PILL_PADDING: f32 = 8.0;
    const PILL_GAP: f32 = 5.0;
    const PILL_DOT: f32 = 5.0;
    const PILL_FILL_MIX: f32 = 0.2;
    const PILL_LINE_MIX: f32 = 0.55;
    /// Paper: Scene switcher list sheet, the room above the actions, and one action's height and room left and right.
    const FOOTER_TOP: f32 = 8.0;
    const ACTION: f32 = 32.0;
    const ACTION_PADDING: f32 = 10.0;
    /// Paper: Scene switcher list sheet, the width of an action's icon slot and of its key chord.
    const ACTION_ICON: f32 = 14.0;
    const ACTION_SHORTCUT: f32 = 100.0;
    const HEAD: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();
    const HINT: TextStyle = TextStyle::new(TypeSize::Micro, FontFace::Regular);
    const NAME: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);
    const NAME_ACTIVE: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);
    const SMALL: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);

    pub fn new(view: &'a SceneSwitcherListView) -> Self {
        Self { view }
    }

    /// Shows the list; returns what was clicked.
    pub fn show(&self, ui: &mut Ui) -> Option<SceneSwitcherListAction> {
        self.show_placed(ui).0
    }

    /// Like `show`, with the rect of each row in points, in order: Original, the scenes, the footer actions. Each
    /// is keyed by the action its click gives (`SelectOriginal`, `SelectScene(n)`, `Footer(n)`).
    pub fn show_placed(&self, ui: &mut Ui) -> (Option<SceneSwitcherListAction>, Vec<(SceneSwitcherListAction, Rect)>) {
        let mut places = Vec::new();
        let width = <container::SceneMenu as ContainerToken>::VALUE;
        let (frame, allocated) = ui.allocate_exact_size(vec2(width, self.height()), Sense::hover());
        let base = allocated.id;
        let painter = ui.painter().clone();
        let fill = <color::BgRaised as ColorToken>::VALUE;
        let line = Some(<color::LineStrong as ColorToken>::VALUE);
        Draw::surface(&painter, frame, Corner::Panel, fill, line, Elevation::Floating);
        let border = <spacing::Border as SpacingToken>::VALUE;
        let inner = Rect::from_min_max(
            pos2(frame.left() + border + Self::PADDING, frame.top() + border),
            pos2(frame.right() - border - Self::PADDING, frame.bottom() - border),
        );
        let mut action = None;
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let head = Rect::from_min_size(inner.min, vec2(inner.width(), Self::HEADER));
        Self::left(
            ui,
            &self.view.header,
            pos2(head.left() + Self::HEADER_PADDING, head.center().y),
            Self::HEAD,
            faint,
            head.width(),
        );
        Self::right(ui, &self.view.hint, pos2(head.right() - Self::HEADER_PADDING, head.center().y), Self::HINT, faint);

        let mut top = head.bottom();
        let original = Rect::from_min_size(pos2(inner.left(), top), vec2(inner.width(), Self::ROW));
        if self.original_row(ui, original, Self::id(base, 0)) {
            action = Some(SceneSwitcherListAction::SelectOriginal);
        }
        places.push((SceneSwitcherListAction::SelectOriginal, original));
        top = original.bottom() + Self::ROW_GAP;
        for (index, scene) in self.view.scenes.iter().enumerate() {
            let row = Rect::from_min_size(pos2(inner.left(), top), vec2(inner.width(), Self::ROW));
            if let Some(found) = self.scene_row(ui, row, scene, index, Self::id(base, 1 + index)) {
                action = Some(found);
            }
            places.push((SceneSwitcherListAction::SelectScene(index), row));
            top = row.bottom() + Self::ROW_GAP;
        }
        // the gap after the last row is not part of the list
        let list_end = top - Self::ROW_GAP;
        if let Some(found) = self.footer(ui, inner, list_end, base, &mut places) {
            action = Some(found);
        }
        (action, places)
    }

    fn height(&self) -> f32 {
        let border = <spacing::Border as SpacingToken>::VALUE;
        let rows = (1 + self.view.scenes.len()) as f32;
        let list = rows * Self::ROW + (rows - 1.0) * Self::ROW_GAP;
        let actions = self.view.footer.len() as f32;
        let footer = if actions > 0.0 {
            border + Self::FOOTER_TOP + actions * Self::ACTION + (actions - 1.0) * Self::ROW_GAP
        } else {
            0.0
        };
        2.0 * border + Self::HEADER + list + footer + Self::PADDING_BOTTOM
    }

    // the Original row; true when it was clicked
    fn original_row(&self, ui: &mut Ui, row: Rect, id: Id) -> bool {
        let original = &self.view.original;
        let response = ui.interact(row, id, Sense::click());
        Self::paint_row(ui, row, original.active, InteractionState::of(&response, true));
        let dot = pos2(row.left() + Self::ROW_PADDING_LEFT + Self::DOT / 2.0, row.center().y);
        ui.painter().circle_filled(dot, Self::DOT / 2.0, <color::TextDim as ColorToken>::VALUE);
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let detail = Self::right(
            ui,
            &original.detail,
            pos2(row.right() - Self::ROW_PADDING_RIGHT, row.center().y),
            Self::SMALL,
            faint,
        );
        let left = dot.x + Self::DOT / 2.0 + Self::GAP;
        let style = if original.active { Self::NAME_ACTIVE } else { Self::NAME };
        let ink = <color::Text as ColorToken>::VALUE;
        Self::left(
            ui,
            &original.name,
            pos2(left, row.center().y),
            style,
            ink,
            (detail.left() - Self::GAP - left).max(0.0),
        );
        response.on_hover_cursor(CursorIcon::PointingHand).clicked()
    }

    // a scene row: dot, name (and check), range, camera count and the status pill
    fn scene_row(
        &self,
        ui: &mut Ui,
        row: Rect,
        scene: &SceneSwitcherRow,
        index: usize,
        id: Id,
    ) -> Option<SceneSwitcherListAction> {
        let response = ui.interact(row, id, Sense::click());
        Self::paint_row(ui, row, scene.active, InteractionState::of(&response, true));
        let colour = scene.color.color();
        let centre = row.center().y;
        let dot = pos2(row.left() + Self::ROW_PADDING_LEFT + Self::DOT / 2.0, centre);
        ui.painter().circle_filled(dot, Self::DOT / 2.0, colour);
        let name_left = dot.x + Self::DOT / 2.0 + Self::GAP;
        let style = if scene.active { Self::NAME_ACTIVE } else { Self::NAME };
        let ink = <color::Text as ColorToken>::VALUE;
        let name = Self::left(ui, &scene.name, pos2(name_left, centre), style, ink, Self::NAME_COLUMN);
        if scene.active {
            let icon = IconSize::Small.points();
            let slot =
                Rect::from_center_size(pos2(name.right() + Self::CHECK_GAP + icon / 2.0, centre), vec2(icon, icon));
            Icon::Check.paint(ui.painter(), slot, IconSize::Small, <color::AccentHover as ColorToken>::VALUE);
        }
        let status_left = row.right() - Self::ROW_PADDING_RIGHT - Self::STATUS_COLUMN;
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let cameras = Self::right(ui, &scene.cameras, pos2(status_left - Self::GAP, centre), Self::SMALL, faint);
        let range_left = name_left + Self::NAME_COLUMN + Self::GAP;
        let dim = <color::TextDim as ColorToken>::VALUE;
        Self::left(
            ui,
            &scene.range,
            pos2(range_left, centre),
            Self::SMALL,
            dim,
            (cameras.left() - Self::GAP - range_left).max(0.0),
        );
        Self::pill(ui, scene, pos2(row.right() - Self::ROW_PADDING_RIGHT, centre));

        let pointer = response.interact_pointer_pos();
        let secondary = response.secondary_clicked();
        let clicked = response.on_hover_cursor(CursorIcon::PointingHand).clicked();
        match (secondary, pointer) {
            (true, Some(at)) => Some(SceneSwitcherListAction::OpenMenu(index, at)),
            _ if clicked => Some(SceneSwitcherListAction::SelectScene(index)),
            _ => None,
        }
    }

    // the status pill, its right edge at `right`: the scene colour behind a dot and the status words
    fn pill(ui: &Ui, scene: &SceneSwitcherRow, right: Pos2) {
        let painter = ui.painter();
        let colour = scene.color.color();
        let words = UiText::resolve(ui.ctx(), &scene.status);
        let galley = Draw::galley(painter, &words, Self::SMALL, <color::Text as ColorToken>::VALUE, TextFit::Line);
        let width = Self::PILL_PADDING * 2.0 + Self::PILL_DOT + Self::PILL_GAP + galley.size().x;
        let rect = Rect::from_min_size(pos2(right.x - width, right.y - Self::PILL / 2.0), vec2(width, Self::PILL));
        Draw::fill(painter, rect, Corner::Full, colour.gamma_multiply(Self::PILL_FILL_MIX));
        Draw::stroke(painter, rect, Corner::Full, colour.gamma_multiply(Self::PILL_LINE_MIX));
        let dot = pos2(rect.left() + Self::PILL_PADDING + Self::PILL_DOT / 2.0, rect.center().y);
        painter.circle_filled(dot, Self::PILL_DOT / 2.0, colour);
        let at = pos2(dot.x + Self::PILL_DOT / 2.0 + Self::PILL_GAP, rect.center().y - galley.size().y / 2.0);
        painter.galley(at, galley, <color::Text as ColorToken>::VALUE);
    }

    // the line, then one row per action; returns the footer action that was clicked
    fn footer(
        &self,
        ui: &mut Ui,
        inner: Rect,
        list_end: f32,
        base: Id,
        places: &mut Vec<(SceneSwitcherListAction, Rect)>,
    ) -> Option<SceneSwitcherListAction> {
        if self.view.footer.is_empty() {
            return None;
        }
        let dim = <color::TextDim as ColorToken>::VALUE;
        let faint = <color::TextFaint as ColorToken>::VALUE;
        Draw::divider(ui.painter(), inner.x_range(), list_end, <color::Line as ColorToken>::VALUE);
        let mut top = list_end + <spacing::Border as SpacingToken>::VALUE + Self::FOOTER_TOP;
        let mut action = None;
        for (index, item) in self.view.footer.iter().enumerate() {
            let row = Rect::from_min_size(pos2(inner.left(), top), vec2(inner.width(), Self::ACTION));
            places.push((SceneSwitcherListAction::Footer(index), row));
            let response = ui.interact(row, Self::id(base, 1 + self.view.scenes.len() + index), Sense::click());
            let state = InteractionState::of(&response, true);
            Draw::fill(ui.painter(), row, Corner::Control, StateStyle::Clear.fill(state));
            let slot = Rect::from_center_size(
                pos2(row.left() + Self::ACTION_PADDING + Self::ACTION_ICON / 2.0, row.center().y),
                vec2(Self::ACTION_ICON, Self::ACTION_ICON),
            );
            item.icon.paint(ui.painter(), slot, IconSize::Small, dim);
            let left = slot.right() + Self::ACTION_PADDING;
            let shortcut_left = row.right() - Self::ACTION_PADDING - Self::ACTION_SHORTCUT;
            Self::left(
                ui,
                &item.label,
                pos2(left, row.center().y),
                Self::NAME,
                dim,
                (shortcut_left - Self::ACTION_PADDING - left).max(0.0),
            );
            if let Some(shortcut) = &item.shortcut {
                Self::left(ui, shortcut, pos2(shortcut_left, row.center().y), Self::HINT, faint, Self::ACTION_SHORTCUT);
            }
            if response.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                action = Some(SceneSwitcherListAction::Footer(index));
            }
            top = row.bottom() + Self::ROW_GAP;
        }
        action
    }

    // a row's fill: soft accent when active, else clear with the hover ladder
    fn paint_row(ui: &Ui, row: Rect, active: bool, state: InteractionState) {
        let style = if active { StateStyle::Soft } else { StateStyle::Clear };
        Draw::fill(ui.painter(), row, Corner::Control, style.fill(state));
    }

    // a word with its left centre at `at`, cut to `room`; returns its rect
    fn left(ui: &Ui, text: &Text, at: Pos2, style: TextStyle, colour: Color32, room: f32) -> Rect {
        let resolved = UiText::resolve(ui.ctx(), text);
        let galley = Draw::galley(ui.painter(), &resolved, style, colour, TextFit::Truncate(room));
        let rect = Rect::from_min_size(pos2(at.x, at.y - galley.size().y / 2.0), galley.size());
        ui.painter().galley(rect.min, galley, colour);
        rect
    }

    // a word with its right centre at `at`; returns its rect
    fn right(ui: &Ui, text: &Text, at: Pos2, style: TextStyle, colour: Color32) -> Rect {
        let resolved = UiText::resolve(ui.ctx(), text);
        let galley = Draw::galley(ui.painter(), &resolved, style, colour, TextFit::Line);
        let rect = Rect::from_min_size(pos2(at.x - galley.size().x, at.y - galley.size().y / 2.0), galley.size());
        ui.painter().galley(rect.min, galley, colour);
        rect
    }

    // the list's own id, so two lists on one screen do not share their rows
    fn id(base: Id, index: usize) -> Id {
        base.with(StudioUiId::SceneSwitcherList).with(index)
    }
}
