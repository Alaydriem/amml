mod realm_segment;

pub use realm_segment::RealmSegment;

use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, OpacityToken, color, opacity};
use egui::{Align2, Painter, Rect, Sense, Ui, pos2, vec2};

use super::{TimeAxis, TrackHeader};
use crate::message::StudioUiMessage;

/// Paper: `Realm row` (`6IW-0`) and the realm row of `54P-0`. The REALM header cell, then a segment for each realm
/// the replay passes through, in the realm colour at `--opacity-muted`. A segment's name shows only when it fits.
pub struct RealmRow<'a> {
    axis: &'a TimeAxis,
    segments: &'a [RealmSegment],
}

impl<'a> RealmRow<'a> {
    /// Paper: the realm row is 18 high.
    pub const HEIGHT: f32 = 18.0;
    // Paper: segments are 11 high at 3 from the top; the name is 6 in
    const SEGMENT_TOP: f32 = 3.0;
    const SEGMENT_HEIGHT: f32 = 11.0;
    const LABEL_X: f32 = 6.0;
    const HEAD_X: f32 = 14.0;

    pub fn new(axis: &'a TimeAxis, segments: &'a [RealmSegment]) -> Self {
        Self { axis, segments }
    }

    pub fn show(&self, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::HEIGHT), Sense::hover());
        self.paint(ui.painter(), rect);
    }

    pub fn paint(&self, painter: &Painter, rect: Rect) {
        let (header, area) = TrackHeader::split(rect);
        TrackHeader::cell(painter, header, <color::BgApp as ColorToken>::VALUE);
        TrackHeader::head(painter, header, &Text::message(StudioUiMessage::TimelineRealm), Self::HEAD_X);
        painter.rect_filled(area, 0, <color::BgApp as ColorToken>::VALUE);
        Draw::divider(painter, area.x_range(), area.bottom() - 1.0, <color::Line as ColorToken>::VALUE);
        let painter = painter.with_clip_rect(area.intersect(painter.clip_rect()));
        let style = TextStyle::new(TypeSize::Micro, FontFace::Semibold);
        let ink = <color::Text as ColorToken>::VALUE;
        for segment in self.segments {
            let top = area.top() + Self::SEGMENT_TOP;
            let place = Rect::from_min_max(
                pos2(self.axis.x_of(segment.start), top),
                pos2(self.axis.x_of(segment.end), top + Self::SEGMENT_HEIGHT),
            );
            let fill = segment.realm.color().gamma_multiply(<opacity::Muted as OpacityToken>::VALUE);
            Draw::fill(&painter, place, Corner::Small, fill);
            let visible = place.intersect(area);
            let name = UiText::resolve(painter.ctx(), &segment.name);
            let galley = Draw::galley(&painter, &name, style, ink, TextFit::Line);
            let x = visible.left() + Self::LABEL_X;
            if x + galley.size().x + Self::LABEL_X <= visible.right() {
                painter.galley(
                    Align2::LEFT_CENTER.anchor_size(pos2(x, place.center().y), galley.size()).min,
                    galley,
                    ink,
                );
            }
        }
    }
}
