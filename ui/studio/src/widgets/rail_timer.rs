use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, color};
use egui::{Align2, Response, Sense, Ui, pos2, vec2};

use crate::message::StudioUiMessage;
use crate::widgets::{RailTile, RailTimerState};

/// Paper: Entry Point, `Components · capture rail & corner buttons` (`A89-0`), "Timer · live" and "Timer · paused".
/// The capture rail's top block: a REC or PAUSED tag over the elapsed time, a rail tile wide.
pub struct RailTimer;

impl RailTimer {
    /// Paper: 10 above and below, a 15 px tag row, 6 apart, then the `--leading-label` time.
    pub const HEIGHT: f32 = 61.0;
    const PADDING: f32 = 10.0;
    const TAG_ROW: f32 = 15.0;
    const GAP: f32 = 6.0;
    // Paper: the mark is 9 across, 5 from the tag; the paused mark has 1 px corners
    const MARK: f32 = 9.0;
    const MARK_GAP: f32 = 5.0;
    const PAUSED_CORNER: u8 = 1;
    const TAG: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();
    const TIME: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Regular);

    /// "mm:ss" under an hour, "h:mm:ss" from an hour.
    pub fn label(elapsed_ms: u64) -> String {
        let seconds = elapsed_ms / 1000;
        let (hours, minutes, secs) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
        if hours > 0 { format!("{hours}:{minutes:02}:{secs:02}") } else { format!("{minutes:02}:{secs:02}") }
    }

    pub fn show(ui: &mut Ui, state: RailTimerState, elapsed_ms: u64) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(RailTile::SIDE, Self::HEIGHT), Sense::hover());
        let (word, mark, tag_ink, time_ink) = match state {
            RailTimerState::Live => (
                StudioUiMessage::RailRecording,
                <color::Playhead as ColorToken>::VALUE,
                <color::PlayheadHover as ColorToken>::VALUE,
                <color::Text as ColorToken>::VALUE,
            ),
            RailTimerState::Paused => (
                StudioUiMessage::RailPaused,
                <color::Warning as ColorToken>::VALUE,
                <color::Warning as ColorToken>::VALUE,
                <color::TextDim as ColorToken>::VALUE,
            ),
        };
        let painter = ui.painter();
        Draw::fill(painter, rect, Corner::Panel, <color::Pill as ColorToken>::VALUE);
        Draw::stroke(painter, rect, Corner::Panel, mark);
        let tag = UiText::resolve(ui.ctx(), &Text::message(word));
        let galley = Draw::galley(painter, &tag, Self::TAG, tag_ink, TextFit::Line);
        let row = Self::MARK + Self::MARK_GAP + galley.size().x;
        let y = rect.top() + Self::PADDING + Self::TAG_ROW / 2.0;
        let left = rect.center().x - row / 2.0;
        let dot = egui::Rect::from_center_size(pos2(left + Self::MARK / 2.0, y), vec2(Self::MARK, Self::MARK));
        match state {
            RailTimerState::Live => {
                painter.circle_filled(dot.center(), Self::MARK / 2.0, mark);
            }
            RailTimerState::Paused => {
                painter.rect_filled(dot, Self::PAUSED_CORNER, mark);
            }
        }
        let at = Align2::LEFT_CENTER.anchor_size(pos2(dot.right() + Self::MARK_GAP, y), galley.size());
        painter.galley(at.min, galley, tag_ink);
        let time = Self::label(elapsed_ms);
        let top = rect.top() + Self::PADDING + Self::TAG_ROW + Self::GAP;
        let galley = Draw::galley(painter, &time, Self::TIME, time_ink, TextFit::Line);
        let at = Align2::CENTER_TOP.anchor_size(pos2(rect.center().x, top), galley.size());
        painter.galley(at.min, galley, time_ink);
        response
    }
}
