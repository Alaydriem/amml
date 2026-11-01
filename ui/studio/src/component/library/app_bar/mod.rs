mod library_app_bar_action;
mod library_app_bar_tab;
mod library_app_bar_view;

pub use library_app_bar_action::LibraryAppBarAction;
pub use library_app_bar_tab::LibraryAppBarTab;
pub use library_app_bar_view::LibraryAppBarView;

use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::Icon;
use amml_ui::state::InteractionState;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{ButtonKind, Leading, TextButton};
use egui::{CursorIcon, Id, Rect, Sense, Ui, pos2, vec2};

use crate::id::StudioUiId;

/// Paper: Editor components, `Editor / Library app bar` (`G4L-0`). The brand tile and name, the Library and
/// Settings tabs with the accent line under the active one, and the Refresh, Open file and Back buttons at the
/// right. It takes the width left and is `HEIGHT` high.
pub struct LibraryAppBar<'a> {
    view: &'a LibraryAppBarView,
}

impl<'a> LibraryAppBar<'a> {
    /// Paper: Library app bar sheet, the bar height.
    pub const HEIGHT: f32 = 56.0;
    /// Paper: Library app bar sheet, the room left and right of the parts.
    const PADDING: f32 = 20.0;
    /// Paper: Library app bar sheet, the brand tile side.
    const TILE: f32 = 26.0;
    /// Paper: Library app bar sheet, the gap between the tile and the name.
    const NAME_GAP: f32 = 12.0;
    /// Paper: Library app bar sheet, the least width of the name block, gap included.
    const NAME_BLOCK: f32 = 110.0;
    /// Paper: Library app bar sheet, the room left and right of a tab word.
    const TAB_PADDING: f32 = 10.0;
    /// Paper: Library app bar sheet, the gap between two tabs.
    const TAB_GAP: f32 = 14.0;
    /// Paper: Library app bar sheet, the button height.
    const BUTTON: f32 = 32.0;
    /// Paper: Library app bar sheet, the gap between buttons.
    const BUTTON_GAP: f32 = 10.0;
    const MONOGRAM: TextStyle = TextStyle::new(TypeSize::Micro, FontFace::Semibold);
    const NAME: TextStyle = TextStyle::new(TypeSize::Body, FontFace::Semibold);
    const TAB: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);

    pub fn new(view: &'a LibraryAppBarView) -> Self {
        Self { view }
    }

    /// Shows the bar; returns the action of the control that was clicked.
    pub fn show(&self, ui: &mut Ui) -> Option<LibraryAppBarAction> {
        let (rect, frame) = ui.allocate_exact_size(vec2(ui.available_width(), Self::HEIGHT), Sense::hover());
        let painter = ui.painter().clone();
        Draw::divider(&painter, rect.x_range(), rect.bottom() - 1.0, <color::Line as ColorToken>::VALUE);
        let centre = rect.center().y;
        let tile = Rect::from_center_size(
            pos2(rect.left() + Self::PADDING + Self::TILE / 2.0, centre),
            vec2(Self::TILE, Self::TILE),
        );
        Draw::fill(&painter, tile, Corner::Control, <color::Accent as ColorToken>::VALUE);
        let on_accent = <color::OnAccent as ColorToken>::VALUE;
        Self::word(ui, &self.view.monogram, tile.center(), Self::MONOGRAM, on_accent, true);
        let name_left = tile.right() + Self::NAME_GAP;
        let name = Self::word(
            ui,
            &self.view.name,
            pos2(name_left, centre),
            Self::NAME,
            <color::Text as ColorToken>::VALUE,
            false,
        );
        let block = Self::NAME_BLOCK.max(Self::NAME_GAP + name.width());
        let mut clicked = None;
        let mut x = tile.right() + block;
        for tab in [LibraryAppBarTab::Library, LibraryAppBarTab::Settings] {
            let label = self.label(tab);
            let resolved = UiText::resolve(ui.ctx(), label);
            let word = Draw::measure(ui.ctx(), &resolved, Self::TAB).x;
            let cell =
                Rect::from_min_size(pos2(x, rect.top()), vec2(Self::TAB_PADDING * 2.0 + word.ceil(), Self::HEIGHT));
            let response = ui.interact(cell, Self::id(frame.id, tab as usize), Sense::click());
            let active = tab == self.view.tab;
            let ink = if active { <color::Text as ColorToken>::VALUE } else { <color::TextDim as ColorToken>::VALUE };
            let text_left = cell.left() + Self::TAB_PADDING;
            let drawn = Self::word(ui, label, pos2(text_left, centre), Self::TAB, ink, false);
            if active {
                let line = <spacing::BorderStrong as SpacingToken>::VALUE;
                let under =
                    Rect::from_min_max(pos2(drawn.left(), rect.bottom() - line), pos2(drawn.right(), rect.bottom()));
                Draw::fill(&painter, under, Corner::Full, <color::Accent as ColorToken>::VALUE);
            } else if response.clicked() {
                clicked = Some(LibraryAppBarAction::SelectTab(tab));
            }
            response.on_hover_cursor(CursorIcon::PointingHand);
            x = cell.right() + Self::TAB_GAP;
        }
        self.buttons(ui, rect, frame.id, &mut clicked);
        clicked
    }

    fn label(&self, tab: LibraryAppBarTab) -> &Text {
        match tab {
            LibraryAppBarTab::Library => &self.view.library,
            LibraryAppBarTab::Settings => &self.view.settings,
        }
    }

    // the buttons, laid from the right edge inwards: Back, Open file, Refresh
    fn buttons(&self, ui: &mut Ui, bar: Rect, base: Id, clicked: &mut Option<LibraryAppBarAction>) {
        let mut right = bar.right() - Self::PADDING;
        let mut parts = vec![
            (ButtonKind::Ghost, &self.view.refresh, Icon::Refresh, LibraryAppBarAction::Refresh),
            (ButtonKind::Outline, &self.view.open_file, Icon::Folder, LibraryAppBarAction::OpenFile),
        ];
        if let Some(back) = &self.view.back {
            parts.push((ButtonKind::Secondary, back, Icon::Back, LibraryAppBarAction::Back));
        }
        for (index, (kind, text, icon, action)) in parts.into_iter().enumerate().rev() {
            let button = TextButton::new(kind, text.clone()).leading(Leading::Icon(icon));
            let width = button.desired_width(ui.ctx());
            let cell = Rect::from_min_size(
                pos2(right - width, bar.center().y - Self::BUTTON / 2.0),
                vec2(width, Self::BUTTON),
            );
            let response = ui.interact(cell, Self::id(base, 2 + index), Sense::click());
            button.paint(ui.painter(), cell, InteractionState::of(&response, true));
            if response.clicked() {
                *clicked = Some(action);
            }
            response.on_hover_cursor(CursorIcon::PointingHand);
            right = cell.left() - Self::BUTTON_GAP;
        }
    }

    // one word at `at` (its left centre, or its centre when `centred`); returns its rect
    fn word(ui: &Ui, text: &Text, at: egui::Pos2, style: TextStyle, colour: egui::Color32, centred: bool) -> Rect {
        let resolved = UiText::resolve(ui.ctx(), text);
        let galley = Draw::galley(ui.painter(), &resolved, style, colour, TextFit::Line);
        let size = galley.size();
        let min = if centred { at - size / 2.0 } else { pos2(at.x, at.y - size.y / 2.0) };
        ui.painter().galley(min, galley, colour);
        Rect::from_min_size(min, size)
    }

    // the bar's own id, so two bars on one screen do not share their controls
    fn id(base: Id, index: usize) -> Id {
        base.with(StudioUiId::LibraryAppBar).with(index)
    }
}
