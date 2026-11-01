mod ruler_action;

pub use ruler_action::RulerAction;

use amml_i18n::text::Text;
use amml_ui::draw::{Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::glyph::Glyph;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Align2, Painter, Rect, Sense, Ui, pos2, vec2};

use super::{TimeAxis, Timecode, TrackHeader};
use crate::glyph::CheckpointTick;
use crate::tokens::color as studio;

/// Paper: `Ruler` (`5Y5-0`) and the ruler row of `54P-0`. The header cell with the track list's title, then the
/// time ticks, a label on each major tick, the checkpoints, and a band for the recorded parts. The tick step is the
/// smallest one whose labels do not overlap.
pub struct Ruler<'a> {
    axis: &'a TimeAxis,
    length: f64,
    title: Option<&'a Text>,
    checkpoints: &'a [f64],
    recorded: &'a [(f64, f64)],
}

impl<'a> Ruler<'a> {
    /// Paper: the ruler row is 48 high.
    pub const HEIGHT: f32 = 48.0;
    // the major tick steps the ruler picks from, in seconds
    const STEPS: [f64; 19] = [
        0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0, 900.0, 1800.0, 3600.0,
        7200.0, 14_400.0,
    ];
    // Paper: major ticks are 10 high, half steps 6, quarter steps 3, all down to the bottom border
    const MAJOR: f32 = 10.0;
    const HALF: f32 = 6.0;
    const QUARTER: f32 = 3.0;
    // quarter ticks closer than this are left out
    const MINOR_SPACING: f32 = 4.0;
    // Paper: a label is 4.6 right of its tick, at 28 from the top; code keeps 16 between labels
    const LABEL_OFFSET: f32 = 4.6;
    const LABEL_TOP: f32 = 28.0;
    const LABEL_GAP: f32 = 16.0;
    // Paper: checkpoints are centred 43 from the top
    const CHECKPOINT_Y: f32 = 43.0;
    const TITLE_X: f32 = 14.0;
    const LABEL: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);

    pub fn new(axis: &'a TimeAxis, length: f64) -> Self {
        Self { axis, length, title: None, checkpoints: &[], recorded: &[] }
    }

    /// The header cell's title, as in "Tracks · Arrival".
    pub fn title(mut self, title: &'a Text) -> Self {
        self.title = Some(title);
        self
    }

    pub fn checkpoints(mut self, checkpoints: &'a [f64]) -> Self {
        self.checkpoints = checkpoints;
        self
    }

    /// The parts of the replay the user recorded, as `(start, end)` seconds. Painted as a band behind everything else.
    pub fn recorded(mut self, ranges: &'a [(f64, f64)]) -> Self {
        self.recorded = ranges;
        self
    }

    pub fn show(&self, ui: &mut Ui) -> Option<RulerAction> {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::HEIGHT), Sense::hover());
        self.show_at(ui, rect)
    }

    /// Shows the ruler in `rect`.
    pub fn show_at(&self, ui: &mut Ui, rect: Rect) -> Option<RulerAction> {
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        self.paint(ui.painter(), rect);
        let at = response.interact_pointer_pos()?;
        // a press that began in the header cell gives nothing, however far it is dragged
        let origin = Self::press_origin(ui, &response).unwrap_or(at);
        if origin.x < rect.left() + TrackHeader::WIDTH {
            return None;
        }
        let seconds = self.axis.seconds_at(at.x).clamp(0.0, self.length);
        if response.secondary_clicked() {
            Some(RulerAction::OpenMenu(seconds, at))
        } else if response.drag_stopped() || response.clicked() {
            Some(RulerAction::Seek(seconds))
        } else if response.dragged() {
            Some(RulerAction::Scrub(seconds))
        } else {
            None
        }
    }

    // where the press began; the pointer state forgets it on the frame of the release, so it is kept in between
    fn press_origin(ui: &Ui, response: &egui::Response) -> Option<egui::Pos2> {
        let id = response.id;
        if let Some(origin) = ui.input(|state| state.pointer.press_origin()) {
            ui.data_mut(|data| data.insert_temp(id, origin));
            return Some(origin);
        }
        let origin = ui.data(|data| data.get_temp::<egui::Pos2>(id));
        if response.drag_stopped() || response.clicked() {
            ui.data_mut(|data| data.remove::<egui::Pos2>(id));
        }
        origin
    }

    pub fn paint(&self, painter: &Painter, rect: Rect) {
        let (header, area) = TrackHeader::split(rect);
        TrackHeader::cell(painter, header, <color::BgPanel as ColorToken>::VALUE);
        if let Some(title) = self.title {
            TrackHeader::head(painter, header, title, Self::TITLE_X);
        }
        painter.rect_filled(area, 0, <color::BgPanel as ColorToken>::VALUE);
        Draw::divider(painter, area.x_range(), area.bottom() - 1.0, <color::Line as ColorToken>::VALUE);
        let painter = painter.with_clip_rect(area.intersect(painter.clip_rect()));
        let step = self.step(painter.ctx());
        self.band(&painter, area);
        self.ticks(&painter, area, step);
        for &seconds in self.checkpoints {
            let x = self.axis.x_of(seconds);
            if (area.left()..=area.right()).contains(&x) {
                let tick = Rect::from_center_size(
                    pos2(x, area.top() + Self::CHECKPOINT_Y),
                    vec2(CheckpointTick::WIDTH, CheckpointTick::HEIGHT),
                );
                CheckpointTick.paint(&painter, tick, <studio::Checkpoint as ColorToken>::VALUE);
            }
        }
    }

    /// The seconds between major ticks: the smallest step whose labels keep `LABEL_GAP` apart.
    pub fn step(&self, ctx: &egui::Context) -> f64 {
        let long = self.length >= 3600.0;
        Self::STEPS
            .into_iter()
            .find(|&step| {
                let label = Draw::measure(ctx, &Timecode::widest(step, long), Self::LABEL).x;
                (step / self.axis.seconds_per_point) as f32 >= label + Self::LABEL_OFFSET + Self::LABEL_GAP
            })
            .unwrap_or(Self::STEPS[Self::STEPS.len() - 1])
    }

    /// Paper `Recorded segment`: an accent-soft fill over the ruler's height, and an accent edge along its bottom.
    fn band(&self, painter: &Painter, area: Rect) {
        let fill = <color::AccentSoft as ColorToken>::VALUE;
        let edge = <color::Accent as ColorToken>::VALUE;
        let height = <spacing::Indicator as SpacingToken>::VALUE;
        for &(start, end) in self.recorded {
            let span =
                Rect::from_x_y_ranges(self.axis.x_of(start)..=self.axis.x_of(end), area.y_range()).intersect(area);
            if span.width() <= 0.0 {
                continue;
            }
            painter.rect_filled(span, 0, fill);
            painter.rect_filled(Rect::from_min_max(pos2(span.left(), span.bottom() - height), span.max), 0, edge);
        }
    }

    fn ticks(&self, painter: &Painter, area: Rect, step: f64) {
        let minor = step / 4.0;
        let quarters = (minor / self.axis.seconds_per_point) as f32 >= Self::MINOR_SPACING;
        let unit = if quarters { minor } else { step };
        let first = (self.axis.seconds_at(area.left()).max(0.0) / unit).floor() as i64;
        let last = (self.axis.seconds_at(area.right()).min(self.length) / unit).ceil() as i64;
        let long = self.length >= 3600.0;
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let strong = <color::LineStrong as ColorToken>::VALUE;
        let bottom = area.bottom() - 1.0;
        for index in first.max(0)..=last {
            let seconds = index as f64 * unit;
            if seconds > self.length {
                break;
            }
            let x = self.axis.x_of(seconds).round() + 0.5;
            let quarter = if quarters { index % 4 } else { 0 };
            let (height, ink) = match quarter {
                0 => (Self::MAJOR, faint),
                2 => (Self::HALF, strong),
                _ => (Self::QUARTER, strong),
            };
            painter.line_segment([pos2(x, bottom - height), pos2(x, bottom)], egui::Stroke::new(1.0, ink));
            if quarter == 0 {
                let label = Timecode::label(seconds, step, long);
                let galley = Draw::galley(painter, &label, Self::LABEL, faint, TextFit::Line);
                let at = Align2::LEFT_TOP
                    .anchor_size(pos2(x + Self::LABEL_OFFSET, area.top() + Self::LABEL_TOP), galley.size());
                if at.left() >= area.left() && at.right() <= area.right() {
                    painter.galley(at.min, galley, faint);
                }
            }
        }
    }
}
