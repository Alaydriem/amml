mod player_row_action;
mod player_row_view;

pub use player_row_action::PlayerRowAction;
pub use player_row_view::PlayerRowView;

use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::state::{InteractionState, StateStyle};
use amml_ui::text::UiText;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, color};
use egui::{CursorIcon, Rect, Sense, Ui, pos2, vec2};

/// Paper: Editor components, `Editor / Player row` (`G7X-0`). A row of the Players tab: the avatar tile, the name and
/// the distance from the camera, a Follow button and a Go to button. A hovered row has a faded hover fill; the
/// followed player's row is soft accent with an accent button. It takes the width left.
pub struct PlayerRow<'a> {
    view: &'a PlayerRowView,
    hovered: bool,
}

impl<'a> PlayerRow<'a> {
    /// Paper: Player row sheet, the row height.
    const HEIGHT: f32 = 54.0;
    /// Paper: Player row sheet, the room left and right of the parts.
    const PADDING_LEFT: f32 = 10.0;
    const PADDING_RIGHT: f32 = 8.0;
    /// Paper: Player row sheet, the gap between the parts of the row.
    const GAP: f32 = 12.0;
    /// Paper: Player row sheet, the avatar tile side.
    const TILE: f32 = 32.0;
    /// Paper: Player row sheet, the gap between the name and the distance.
    const LINE_GAP: f32 = 3.0;
    /// Paper: Player row sheet, the button height, its room left and right, and the gap after its icon.
    const BUTTON: f32 = 26.0;
    const FOLLOW_PADDING: f32 = 10.0;
    const GO_PADDING: f32 = 8.0;
    const ICON_GAP: f32 = 6.0;
    /// Paper: Player row sheet, how much of the hover fill a hovered row shows, and of the accent its line.
    const HOVER_MIX: f32 = 0.7;
    const FOLLOWING_LINE_MIX: f32 = 0.4;
    const INITIAL: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Semibold);
    const NAME: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);
    const DISTANCE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);
    const BUTTON_WORD: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);
    const BUTTON_WORD_ON: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Semibold);

    pub fn new(view: &'a PlayerRowView) -> Self {
        Self { view, hovered: false }
    }

    /// Draws the hover look without the pointer, for the gallery.
    pub fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    /// Shows the row; returns the action of the button that was clicked.
    pub fn show(&self, ui: &mut Ui) -> Option<PlayerRowAction> {
        let (rect, row) = ui.allocate_exact_size(vec2(ui.available_width(), Self::HEIGHT), Sense::hover());
        let painter = ui.painter().clone();
        self.paint_row(&painter, rect, row.hovered() || self.hovered);
        let centre = rect.center().y;
        let tile = Rect::from_center_size(
            pos2(rect.left() + Self::PADDING_LEFT + Self::TILE / 2.0, centre),
            vec2(Self::TILE, Self::TILE),
        );
        Draw::fill(&painter, tile, Corner::Control, <color::BgField as ColorToken>::VALUE);
        Draw::stroke(&painter, tile, Corner::Control, <color::LineStrong as ColorToken>::VALUE);
        let dim = <color::TextDim as ColorToken>::VALUE;
        Self::paint_text(ui, &self.view.initial, tile.center(), Self::INITIAL, dim);

        // the buttons are laid from the right edge inwards
        let go_to = UiText::resolve(ui.ctx(), &self.view.go_to);
        let go_width = Draw::measure(ui.ctx(), &go_to, Self::BUTTON_WORD).x.ceil() + Self::GO_PADDING * 2.0;
        let go_rect = Rect::from_min_size(
            pos2(rect.right() - Self::PADDING_RIGHT - go_width, centre - Self::BUTTON / 2.0),
            vec2(go_width, Self::BUTTON),
        );
        let follow_style = if self.view.following { Self::BUTTON_WORD_ON } else { Self::BUTTON_WORD };
        let follow = UiText::resolve(ui.ctx(), &self.view.follow);
        let icon = IconSize::Small.points();
        let follow_width = Self::FOLLOW_PADDING * 2.0
            + icon
            + Self::ICON_GAP
            + Draw::measure(ui.ctx(), &follow, follow_style).x.ceil();
        let follow_rect = Rect::from_min_size(
            pos2(go_rect.left() - Self::GAP - follow_width, centre - Self::BUTTON / 2.0),
            vec2(follow_width, Self::BUTTON),
        );

        let mut action = None;
        let go = ui.interact(go_rect, row.id.with(0), Sense::click());
        if go.hovered() {
            let state = InteractionState::of(&go, true);
            Draw::fill(&painter, go_rect, Corner::Control, StateStyle::Clear.fill(state));
        }
        let ink = <color::Text as ColorToken>::VALUE;
        Self::paint_text(ui, &self.view.go_to, go_rect.center(), Self::BUTTON_WORD, ink);
        if go.on_hover_cursor(CursorIcon::PointingHand).clicked() {
            action = Some(PlayerRowAction::GoTo);
        }
        let toggle = ui.interact(follow_rect, row.id.with(1), Sense::click());
        self.paint_follow(ui, follow_rect, InteractionState::of(&toggle, true), follow_style);
        if toggle.on_hover_cursor(CursorIcon::PointingHand).clicked() {
            action = Some(PlayerRowAction::ToggleFollow);
        }

        let left = tile.right() + Self::GAP;
        let room = (follow_rect.left() - Self::GAP - left).max(0.0);
        self.paint_lines(ui, left, centre, room);
        action
    }

    // the row's own fill: none at rest, the faded hover fill under the pointer, soft accent with a line when followed
    fn paint_row(&self, painter: &egui::Painter, rect: Rect, hovered: bool) {
        if self.view.following {
            let line = <color::Accent as ColorToken>::VALUE.gamma_multiply(Self::FOLLOWING_LINE_MIX);
            let fill = <color::AccentSoft as ColorToken>::VALUE;
            Draw::surface(painter, rect, Corner::Control, fill, Some(line), Elevation::Flat);
        } else if hovered {
            let fill = <color::BgHover as ColorToken>::VALUE.gamma_multiply(Self::HOVER_MIX);
            Draw::fill(painter, rect, Corner::Control, fill);
        }
    }

    // the Follow button: a camera icon and the word; the accent fill while following
    fn paint_follow(&self, ui: &Ui, rect: Rect, state: InteractionState, style: TextStyle) {
        let painter = ui.painter();
        let (fill, ink) = if self.view.following {
            (StateStyle::Accent.fill(state), <color::OnAccent as ColorToken>::VALUE)
        } else {
            (StateStyle::Field.fill(state), <color::Text as ColorToken>::VALUE)
        };
        Draw::fill(painter, rect, Corner::Control, fill);
        let icon = IconSize::Small.points();
        let slot = Rect::from_center_size(
            pos2(rect.left() + Self::FOLLOW_PADDING + icon / 2.0, rect.center().y),
            vec2(icon, icon),
        );
        Icon::Camera.paint(painter, slot, IconSize::Small, ink);
        let word = UiText::resolve(ui.ctx(), &self.view.follow);
        let galley = Draw::galley(painter, &word, style, ink, TextFit::Line);
        painter.galley(pos2(slot.right() + Self::ICON_GAP, rect.center().y - galley.size().y / 2.0), galley, ink);
    }

    // the name over the distance, centred on the row's middle
    fn paint_lines(&self, ui: &Ui, left: f32, centre: f32, room: f32) {
        let painter = ui.painter();
        let name = UiText::resolve(ui.ctx(), &self.view.name);
        let distance = UiText::resolve(ui.ctx(), &self.view.distance);
        let text = <color::Text as ColorToken>::VALUE;
        let dim = <color::TextDim as ColorToken>::VALUE;
        let name = Draw::galley(painter, &name, Self::NAME, text, TextFit::Truncate(room));
        let distance = Draw::galley(painter, &distance, Self::DISTANCE, dim, TextFit::Truncate(room));
        let top = centre - (name.size().y + Self::LINE_GAP + distance.size().y) / 2.0;
        let name_height = name.size().y;
        painter.galley(pos2(left, top), name, text);
        painter.galley(pos2(left, top + name_height + Self::LINE_GAP), distance, dim);
    }

    // one word centred on `at`
    fn paint_text(ui: &Ui, text: &Text, at: egui::Pos2, style: TextStyle, colour: egui::Color32) {
        let resolved = UiText::resolve(ui.ctx(), text);
        let galley = Draw::galley(ui.painter(), &resolved, style, colour, TextFit::Line);
        ui.painter().galley(at - galley.size() / 2.0, galley, colour);
    }
}
