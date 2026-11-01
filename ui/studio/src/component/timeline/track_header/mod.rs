mod track_header_action;
mod track_header_view;

pub use track_header_action::TrackHeaderAction;
pub use track_header_view::TrackHeaderView;

use amml_i18n::text::{Args, Text};
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, color};
use egui::{Align2, Color32, CursorIcon, Painter, Rect, Sense, Ui, pos2, vec2};

use crate::message::StudioUiMessage;

/// Paper: `Track headers` (`6OM-0`). The left cell of a lane: the track colour stripe, the show and lock buttons,
/// the name and the key count. It also draws the left cells of the other timeline rows.
pub struct TrackHeader<'a> {
    view: &'a TrackHeaderView,
}

impl<'a> TrackHeader<'a> {
    /// Paper: the track header column is 222 wide.
    pub const WIDTH: f32 = 222.0;
    // Paper: a 3 px stripe; the eye and lock buttons are 20 square at x 10 and 32, 22 down; the name at x 58
    const STRIPE: f32 = 3.0;
    const BUTTON: f32 = 20.0;
    const EYE_X: f32 = 10.0;
    const LOCK_X: f32 = 32.0;
    const NAME_X: f32 = 58.0;
    // Paper: the count pill is 18 high, 12 from the right, with 6 each side
    const PILL_HEIGHT: f32 = 18.0;
    const PILL_RIGHT: f32 = 12.0;
    const PILL_PADDING: f32 = 6.0;
    const PILL_GAP: f32 = 8.0;
    const HEAD: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();

    pub fn new(view: &'a TrackHeaderView) -> Self {
        Self { view }
    }

    /// Shows the header in `rect` (one lane high, `WIDTH` wide).
    pub fn show_at(&self, ui: &mut Ui, rect: Rect) -> Option<TrackHeaderAction> {
        let row = ui.allocate_rect(Self::visible(ui, rect), Sense::click());
        let eye = Self::visible(ui, self.button_rect(rect, Self::EYE_X));
        let lock = Self::visible(ui, self.button_rect(rect, Self::LOCK_X));
        let (show_word, lock_word) = (
            if self.view.hidden { StudioUiMessage::ShowTrack } else { StudioUiMessage::HideTrack },
            if self.view.locked { StudioUiMessage::UnlockTrack } else { StudioUiMessage::LockTrack },
        );
        let eye_response = ui
            .allocate_rect(eye, Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(UiText::resolve(ui.ctx(), &Text::message(show_word)));
        let lock_response = ui
            .allocate_rect(lock, Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(UiText::resolve(ui.ctx(), &Text::message(lock_word)));
        let fill = if self.view.selected {
            <color::AccentSoft as ColorToken>::VALUE
        } else if row.hovered() {
            <color::BgRaised as ColorToken>::VALUE
        } else {
            <color::BgPanel as ColorToken>::VALUE
        };
        self.paint(ui.painter(), rect, fill);
        if eye_response.clicked() {
            Some(TrackHeaderAction::ToggleHidden)
        } else if lock_response.clicked() {
            Some(TrackHeaderAction::ToggleLocked)
        } else if row.clicked() {
            Some(TrackHeaderAction::Select)
        } else {
            None
        }
    }

    /// Paints the header in `rect` on `fill`.
    pub fn paint(&self, painter: &Painter, rect: Rect, fill: Color32) {
        Self::cell(painter, rect, fill);
        let stripe = Rect::from_min_size(rect.min, vec2(Self::STRIPE, rect.height() - 1.0));
        painter.rect_filled(stripe, 0, self.view.color.color());
        let (dim, faint) = (<color::TextDim as ColorToken>::VALUE, <color::TextFaint as ColorToken>::VALUE);
        let (eye, eye_ink) = if self.view.hidden { (Icon::Hide, faint) } else { (Icon::Show, dim) };
        eye.paint(painter, self.button_rect(rect, Self::EYE_X), IconSize::Small, eye_ink);
        let lock_ink = if self.view.locked { <color::Warning as ColorToken>::VALUE } else { faint };
        Icon::Lock.paint(painter, self.button_rect(rect, Self::LOCK_X), IconSize::Small, lock_ink);
        let pill_style = TextStyle::new(TypeSize::Micro, FontFace::Regular);
        let count = Text::format(StudioUiMessage::Count, Args::new().uint("count", self.view.keys as u64));
        let count = UiText::resolve(painter.ctx(), &count);
        let galley = Draw::galley(painter, &count, pill_style, dim, TextFit::Line);
        let pill = Rect::from_min_size(
            pos2(
                rect.right() - Self::PILL_RIGHT - galley.size().x - Self::PILL_PADDING * 2.0,
                rect.center().y - Self::PILL_HEIGHT / 2.0,
            ),
            vec2(galley.size().x + Self::PILL_PADDING * 2.0, Self::PILL_HEIGHT),
        );
        Draw::fill(painter, pill, Corner::Full, <color::BgField as ColorToken>::VALUE);
        let text = Rect::from_center_size(pill.center(), galley.size());
        painter.galley(text.min, galley, dim);
        let face = if self.view.selected { FontFace::Semibold } else { FontFace::Regular };
        let ink = if self.view.hidden { faint } else { <color::Text as ColorToken>::VALUE };
        let x = rect.left() + Self::NAME_X;
        let room = (pill.left() - Self::PILL_GAP - x).max(0.0);
        let name = TextStyle::new(TypeSize::Help, face);
        Draw::text(
            painter,
            pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            &self.view.name,
            name,
            ink,
            TextFit::Truncate(room),
        );
    }

    /// The part of `rect` inside `ui`'s clip rect: a lane row scrolled half out of view takes only the room it shows,
    /// so the rows under the lanes' viewport do not grow the timeline. Empty (at the clip's corner) when none shows.
    pub fn visible(ui: &Ui, rect: Rect) -> Rect {
        let clip = ui.clip_rect();
        let shown = rect.intersect(clip);
        if shown.is_positive() { shown } else { Rect::from_min_size(clip.min, egui::Vec2::ZERO) }
    }

    /// The header cell of a timeline row in `rect` and the time area right of it.
    pub fn split(rect: Rect) -> (Rect, Rect) {
        let header = Rect::from_min_size(rect.min, vec2(Self::WIDTH.min(rect.width()), rect.height()));
        let area = Rect::from_min_max(pos2(header.right(), rect.top()), rect.max);
        (header, area)
    }

    /// A left cell: `fill`, with the `--color-line` border on its right and bottom.
    pub fn cell(painter: &Painter, rect: Rect, fill: Color32) {
        painter.rect_filled(rect, 0, fill);
        let line = <color::Line as ColorToken>::VALUE;
        painter.vline(rect.right() - 0.5, rect.y_range(), egui::Stroke::new(1.0, line));
        Draw::divider(painter, rect.x_range(), rect.bottom() - 1.0, line);
    }

    /// A row's small-caps head, `x` in from the cell's left, centred on the cell's height. Returns its rect.
    pub fn head(painter: &Painter, cell: Rect, text: &Text, x: f32) -> Rect {
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let room = (cell.width() - x * 2.0).max(0.0);
        Draw::text(
            painter,
            pos2(cell.left() + x, cell.center().y),
            Align2::LEFT_CENTER,
            text,
            Self::HEAD,
            faint,
            TextFit::Truncate(room),
        )
    }

    fn button_rect(&self, rect: Rect, x: f32) -> Rect {
        Rect::from_min_size(
            pos2(rect.left() + x, rect.center().y - Self::BUTTON / 2.0),
            vec2(Self::BUTTON, Self::BUTTON),
        )
    }
}
