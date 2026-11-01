mod scene_band;
mod scene_bands_action;

pub use scene_band::SceneBand;
pub use scene_bands_action::SceneBandsAction;

use amml_i18n::text::{Args, Text};
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::text::UiText;
use amml_ui::tokens::rule::SoftFill;
use amml_ui::tokens::{ColorToken, color};
use egui::{Align2, CursorIcon, Painter, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use super::{TimeAxis, TrackHeader};
use crate::message::StudioUiMessage;
use crate::rule::LabelTint;

/// Paper: `Scene bands` (`6D4-0`) and the scene row of `54P-0`. The header cell (SCENES, the count and "+"), then
/// a band for each scene on one of two rows. The active band has a 2 px border and white words; a band that starts
/// left of the view keeps its words at the view's left edge.
pub struct SceneBands<'a> {
    axis: &'a TimeAxis,
    bands: &'a [SceneBand],
}

impl<'a> SceneBands<'a> {
    /// Paper: the scene row is 52 high.
    pub const HEIGHT: f32 = 52.0;
    // Paper: bands are 21 high on rows 3 and 27 from the top
    const BAND_HEIGHT: f32 = 21.0;
    const ROWS: [f32; 2] = [3.0, 27.0];
    // Paper: the dot is 7, then 6 to the name and 6 more (plus 4) to the range; code pads 8 inside the band
    const DOT: f32 = 7.0;
    const GAP: f32 = 6.0;
    const RANGE_GAP: f32 = 10.0;
    const PADDING: f32 = 8.0;
    // Paper: the active band fill is its colour at 40 %, a resting border at 45 %, the range text at 65 %
    const ACTIVE_ALPHA: u8 = 0x66;
    const BORDER_ALPHA: u8 = 0x73;
    const RANGE_ALPHA: u8 = 0xA6;
    const ACTIVE_BORDER: f32 = 2.0;
    // Paper: the head at (14, 7), the count pill at (78, 6) 16 high, the "+" 22 square at (189, 3)
    const HEAD: [f32; 2] = [14.0, 7.0];
    const COUNT: [f32; 2] = [78.0, 6.0];
    const COUNT_HEIGHT: f32 = 16.0;
    const COUNT_PADDING: f32 = 5.0;
    const ADD: [f32; 2] = [189.0, 3.0];
    const ADD_SIDE: f32 = 22.0;

    pub fn new(axis: &'a TimeAxis, bands: &'a [SceneBand]) -> Self {
        Self { axis, bands }
    }

    pub fn show(&self, ui: &mut Ui) -> Option<SceneBandsAction> {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::HEIGHT), Sense::hover());
        let action = self.show_at(ui, rect);
        // `allocate_rect` moved the cursor to each band; the page goes on under the row
        ui.advance_cursor_after_rect(rect);
        action
    }

    /// Shows the row in `rect`.
    pub fn show_at(&self, ui: &mut Ui, rect: Rect) -> Option<SceneBandsAction> {
        let (header, area) = TrackHeader::split(rect);
        let add =
            Rect::from_min_size(header.min + vec2(Self::ADD[0], Self::ADD[1]), vec2(Self::ADD_SIDE, Self::ADD_SIDE));
        let add_response = ui
            .allocate_rect(add, Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(UiText::resolve(ui.ctx(), &Text::message(StudioUiMessage::AddScene)));
        self.paint_header(ui.painter(), header, add_response.hovered());
        let painter = ui.painter().with_clip_rect(area.intersect(ui.clip_rect()));
        painter.rect_filled(area, 0, <color::BgApp as ColorToken>::VALUE);
        Draw::divider(&painter, area.x_range(), area.bottom() - 1.0, <color::Line as ColorToken>::VALUE);
        let mut action = add_response.clicked().then_some(SceneBandsAction::Add);
        for (index, band) in self.bands.iter().enumerate() {
            let place = self.band_rect(area, band);
            let visible = place.intersect(area);
            if !visible.is_positive() {
                continue;
            }
            let response = ui.allocate_rect(visible, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
            Self::paint_band(&painter, place, area, band, response.hovered());
            if response.secondary_clicked() {
                action = response.interact_pointer_pos().map(|at| SceneBandsAction::OpenMenu(index, at));
            } else if response.clicked() {
                action = Some(SceneBandsAction::Select(index));
            }
        }
        action
    }

    // where a band sits in the time area
    fn band_rect(&self, area: Rect, band: &SceneBand) -> Rect {
        let top = area.top() + Self::ROWS[band.row.min(1)];
        let left = self.axis.x_of(band.start);
        let right = self.axis.x_of(band.end).max(left + 1.0);
        Rect::from_min_max(pos2(left, top), pos2(right, top + Self::BAND_HEIGHT))
    }

    fn paint_header(&self, painter: &Painter, header: Rect, add_hover: bool) {
        TrackHeader::cell(painter, header, <color::BgApp as ColorToken>::VALUE);
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let head = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();
        let at = header.min + vec2(Self::HEAD[0], Self::HEAD[1]);
        let words = Text::message(StudioUiMessage::TimelineScenes);
        Draw::text(painter, at, Align2::LEFT_TOP, &words, head, faint, TextFit::Line);
        let dim = <color::TextDim as ColorToken>::VALUE;
        let count = Text::format(StudioUiMessage::Count, Args::new().uint("count", self.bands.len() as u64));
        let count = UiText::resolve(painter.ctx(), &count);
        let galley =
            Draw::galley(painter, &count, TextStyle::new(TypeSize::Micro, FontFace::Regular), dim, TextFit::Line);
        let pill = Rect::from_min_size(
            header.min + vec2(Self::COUNT[0], Self::COUNT[1]),
            vec2(galley.size().x + Self::COUNT_PADDING * 2.0, Self::COUNT_HEIGHT),
        );
        Draw::fill(painter, pill, Corner::Full, <color::BgField as ColorToken>::VALUE);
        painter.galley(Rect::from_center_size(pill.center(), galley.size()).min, galley, dim);
        let add =
            Rect::from_min_size(header.min + vec2(Self::ADD[0], Self::ADD[1]), vec2(Self::ADD_SIDE, Self::ADD_SIDE));
        if add_hover {
            Draw::fill(painter, add, Corner::Small, <color::BgHover as ColorToken>::VALUE);
        }
        Icon::Add.paint(painter, add, IconSize::Small, dim);
    }

    fn paint_band(painter: &Painter, place: Rect, area: Rect, band: &SceneBand, hovered: bool) {
        let hue = band.color.color();
        let corner = Corner::Control.radius(place);
        let (fill, border, width) = if band.active {
            (hue.gamma_multiply_u8(Self::ACTIVE_ALPHA), hue, Self::ACTIVE_BORDER)
        } else if hovered {
            (SoftFill::of(hue), hue, 1.0)
        } else {
            (SoftFill::of(hue), hue.gamma_multiply_u8(Self::BORDER_ALPHA), 1.0)
        };
        painter.rect_filled(place, corner, fill);
        painter.rect_stroke(place, corner, Stroke::new(width, border), StrokeKind::Inside);
        let (name_style, ink) = if band.active {
            (TextStyle::new(TypeSize::Note, FontFace::Semibold), <color::OnAccent as ColorToken>::VALUE)
        } else {
            (TextStyle::new(TypeSize::Caption, FontFace::Regular), LabelTint::of(hue))
        };
        let y = place.center().y;
        let left = place.left().max(area.left()) + Self::PADDING;
        let right = place.right().min(area.right()) - Self::PADDING;
        let x = left + Self::DOT + Self::GAP;
        let name = UiText::resolve(painter.ctx(), &band.name);
        let whole = Draw::measure(painter.ctx(), &name, name_style).x;
        // Paper: a band too narrow for its name lets the name spill out to its right, over the row
        let spill = x + whole > right;
        let room = if spill { area.right() - Self::PADDING - x } else { right - x }.max(0.0);
        painter.circle_filled(pos2(left + Self::DOT / 2.0, y), Self::DOT / 2.0, hue);
        let galley = Draw::galley(painter, &name, name_style, ink, TextFit::Truncate(room));
        let at = Align2::LEFT_CENTER.anchor_size(pos2(x, y), galley.size());
        painter.galley(at.min, galley, ink);
        let range = UiText::resolve(painter.ctx(), &band.range);
        let style = TextStyle::new(TypeSize::Micro, FontFace::Regular);
        let range_ink = ink.gamma_multiply_u8(Self::RANGE_ALPHA);
        let galley = Draw::galley(painter, &range, style, range_ink, TextFit::Line);
        let x = at.right() + Self::RANGE_GAP;
        // the range shows only when it fits whole
        if x + galley.size().x <= right {
            painter.galley(Align2::LEFT_CENTER.anchor_size(pos2(x, y), galley.size()).min, galley, range_ink);
        }
    }
}
