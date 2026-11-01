mod replay_row_action;
mod replay_row_note;
mod replay_row_view;

pub use replay_row_action::ReplayRowAction;
pub use replay_row_note::ReplayRowNote;
pub use replay_row_view::ReplayRowView;

use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::text::UiText;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, OpacityToken, SpacingToken, color, opacity, spacing};
use amml_ui::widget::MetaSeparator;
use egui::epaint::RectShape;
use egui::{Color32, CornerRadius, CursorIcon, Painter, Pos2, Rect, Sense, Ui, Vec2, pos2, vec2};

use crate::component::library::ReplayPlaceholder;
use crate::tokens::color as studio_color;

/// Paper: Screen / Library (`DX-0`), the rows of the replay list, and `65T-0` for the broken look. A thumbnail with
/// the length on a strip, the name, the facts and the source or the reason the replay is broken. A selected row has
/// the soft accent fill and an accent line; a hovered row has the faded hover fill. It takes the width left and is
/// `HEIGHT` high.
pub struct ReplayRow<'a> {
    view: &'a ReplayRowView,
    hovered: bool,
}

impl<'a> ReplayRow<'a> {
    /// Paper: DX-0, the row height.
    pub const HEIGHT: f32 = 74.0;
    /// Paper: DX-0, the room left and right of the parts, and the gap between the thumbnail and the words.
    const PADDING: f32 = 10.0;
    const GAP: f32 = 14.0;
    /// Paper: DX-0, the thumbnail size and the height of the strip at its foot.
    const THUMB: Vec2 = vec2(89.0, 50.0);
    const STRIP: f32 = 18.0;
    const STRIP_PADDING: f32 = 6.0;
    /// Paper: DX-0, the gap between the lines of words.
    const LINE_GAP: f32 = 4.0;
    /// Paper: DX-0, the room each side of the dot between two facts (Paper writes two spaces).
    const SEPARATOR_GAP: f32 = 7.0;
    /// Paper: 65T-0, the gap between the warning icon and its words.
    const WARNING_GAP: f32 = 6.0;
    /// Paper: DX-0, how much of the hover fill a hovered row shows (the same as the Players tab row).
    const HOVER_MIX: f32 = 0.7;
    /// Paper: 65T-0, how much of the picture a broken replay keeps.
    const BROKEN_PICTURE: f32 = 0.35;
    const NAME: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);
    const FACTS: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);
    const TAG: TextStyle = TextStyle::new(TypeSize::Micro, FontFace::Semibold);
    const LENGTH: TextStyle = TextStyle::new(TypeSize::Micro, FontFace::Regular);
    const WARNING: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold);

    pub fn new(view: &'a ReplayRowView) -> Self {
        Self { view, hovered: false }
    }

    /// Draws the hover look without the pointer, for the gallery.
    pub fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    /// Shows the row; returns what the person did to it.
    pub fn show(&self, ui: &mut Ui) -> Option<ReplayRowAction> {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::HEIGHT), Sense::click());
        let painter = ui.painter().clone();
        self.paint_row(&painter, rect, response.hovered() || self.hovered);
        let border = <spacing::Border as SpacingToken>::VALUE;
        let thumb = Rect::from_min_size(
            pos2(rect.left() + border + Self::PADDING, rect.center().y - Self::THUMB.y / 2.0),
            Self::THUMB,
        );
        self.paint_thumbnail(ui, thumb);
        let left = thumb.right() + Self::GAP;
        let room = (rect.right() - border - Self::PADDING - left).max(0.0);
        self.paint_lines(ui, left, rect.center().y, room);
        let response = response.on_hover_cursor(CursorIcon::PointingHand);
        if response.double_clicked() {
            Some(ReplayRowAction::Open)
        } else if response.clicked() {
            Some(ReplayRowAction::Select)
        } else if response.secondary_clicked() {
            response.interact_pointer_pos().map(ReplayRowAction::Menu)
        } else {
            None
        }
    }

    // nothing at rest, the faded hover fill under the pointer, the soft accent fill and line when selected
    fn paint_row(&self, painter: &Painter, rect: Rect, hovered: bool) {
        if self.view.selected {
            let line = <color::Accent as ColorToken>::VALUE.gamma_multiply(<opacity::Muted as OpacityToken>::VALUE);
            let fill = <color::AccentSoft as ColorToken>::VALUE;
            Draw::surface(painter, rect, Corner::Panel, fill, Some(line), Elevation::Flat);
        } else if hovered {
            let fill = <color::BgHover as ColorToken>::VALUE.gamma_multiply(Self::HOVER_MIX);
            Draw::fill(painter, rect, Corner::Panel, fill);
        }
    }

    // the picture (faint when the replay is broken) and the length on the dark strip at its foot
    fn paint_thumbnail(&self, ui: &Ui, thumb: Rect) {
        let painter = ui.painter();
        let corner = Corner::Control.radius(thumb);
        let keep = if matches!(self.view.note, ReplayRowNote::Broken(_)) { Self::BROKEN_PICTURE } else { 1.0 };
        let tint = Color32::WHITE.gamma_multiply(keep);
        let picture = self.view.thumbnail.unwrap_or_else(|| ReplayPlaceholder::picture(ui.ctx()));
        painter.add(RectShape::filled(thumb, corner, tint).with_texture(picture.texture, picture.cover(thumb.size())));
        let strip = Rect::from_min_max(pos2(thumb.left(), thumb.bottom() - Self::STRIP), thumb.max);
        let strip_corner = CornerRadius { nw: 0, ne: 0, sw: corner.sw, se: corner.se };
        painter.rect_filled(strip, strip_corner, <studio_color::PictureStrip as ColorToken>::VALUE);
        let ink = <color::Text as ColorToken>::VALUE;
        let length = UiText::resolve(ui.ctx(), &self.view.length);
        let galley = Draw::galley(painter, &length, Self::LENGTH, ink, TextFit::Line);
        let at = pos2(strip.right() - Self::STRIP_PADDING - galley.size().x, strip.center().y - galley.size().y / 2.0);
        painter.galley(at, galley, ink);
    }

    // the name, the facts and the note, centred on the row's middle
    fn paint_lines(&self, ui: &Ui, left: f32, centre: f32, room: f32) {
        let painter = ui.painter();
        let text = <color::Text as ColorToken>::VALUE;
        let dim = <color::TextDim as ColorToken>::VALUE;
        let name = UiText::resolve(ui.ctx(), &self.view.name);
        let name = Draw::galley(painter, &name, Self::NAME, text, TextFit::Truncate(room));
        let facts_height = TypeSize::Caption.line_height();
        let note_height = match self.view.note {
            ReplayRowNote::Source(_) => TypeSize::Micro.line_height(),
            ReplayRowNote::Broken(_) => TypeSize::Caption.line_height(),
        };
        let total = name.size().y + Self::LINE_GAP + facts_height + Self::LINE_GAP + note_height;
        let mut top = centre - total / 2.0;
        let name_height = name.size().y;
        painter.galley(pos2(left, top), name, text);
        top += name_height + Self::LINE_GAP;
        Self::paint_facts(ui, &self.view.facts, Self::FACTS, dim, pos2(left, top), (room, facts_height));
        top += facts_height + Self::LINE_GAP;
        match &self.view.note {
            ReplayRowNote::Source(source) => {
                let faint = <color::TextFaint as ColorToken>::VALUE;
                Self::paint_facts(ui, source, Self::TAG, faint, pos2(left, top), (room, note_height));
            }
            ReplayRowNote::Broken(reason) => Self::paint_warning(ui, reason, pos2(left, top), (room, note_height)),
        }
    }

    // `facts` side by side from `at`, a dot between two of them, cut at `room`
    fn paint_facts(ui: &Ui, facts: &[Text], style: TextStyle, colour: Color32, at: Pos2, size: (f32, f32)) {
        let (room, height) = size;
        let painter = ui.painter();
        let mut x = 0.0;
        let last = facts.len().saturating_sub(1);
        for (index, fact) in facts.iter().enumerate() {
            if x >= room {
                break;
            }
            let resolved = UiText::resolve(ui.ctx(), fact);
            let galley = Draw::galley(painter, &resolved, style, colour, TextFit::Truncate(room - x));
            let width = galley.size().x;
            painter.galley(pos2(at.x + x, at.y + (height - galley.size().y) / 2.0), galley, colour);
            x += width;
            if index != last {
                x += Self::SEPARATOR_GAP;
                MetaSeparator::paint(painter, pos2(at.x + x + MetaSeparator::SIDE / 2.0, at.y + height / 2.0));
                x += MetaSeparator::SIDE + Self::SEPARATOR_GAP;
            }
        }
    }

    // the warning icon and the reason, in the danger colour
    fn paint_warning(ui: &Ui, reason: &Text, at: Pos2, size: (f32, f32)) {
        let (room, height) = size;
        let painter = ui.painter();
        let danger = <color::Danger as ColorToken>::VALUE;
        let icon = IconSize::Small.points();
        let slot = Rect::from_center_size(pos2(at.x + icon / 2.0, at.y + height / 2.0), vec2(icon, icon));
        Icon::Info.paint(painter, slot, IconSize::Small, danger);
        let left = slot.right() + Self::WARNING_GAP;
        let resolved = UiText::resolve(ui.ctx(), reason);
        let galley = Draw::galley(
            painter,
            &resolved,
            Self::WARNING,
            danger,
            TextFit::Truncate((room - icon - Self::WARNING_GAP).max(0.0)),
        );
        painter.galley(pos2(left, at.y + (height - galley.size().y) / 2.0), galley, danger);
    }
}
