mod camera_readout;
mod readout_layout;
mod viewer_chips_view;
mod viewer_owner;
mod viewer_source;
mod viewer_tone;

pub use camera_readout::CameraReadout;
pub use readout_layout::ReadoutLayout;
pub use viewer_chips_view::ViewerChipsView;
pub use viewer_owner::ViewerOwner;
pub use viewer_source::ViewerSource;
pub use viewer_tone::ViewerTone;

use amml_i18n::text::{Args, Text};
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Align2, Color32, Painter, Rect, Stroke, StrokeKind, Ui, pos2, vec2};

use super::ViewerPill;
use crate::message::StudioUiMessage;
use crate::tokens::color as studio;

/// Paper: Editor components, `Viewer chips and overlays` (`3TE-0`) as placed on `2LV-0`, `30B-0` and `3I7-0`. The
/// chips over the game picture: the source and lens at the top left, the mode at the top right with a warning under
/// it, the owner tag at the top centre, and the camera readout at the bottom left. While the scene camera drives,
/// the picture gets its frame and corner brackets. Every chip stays inside the viewer rect.
pub struct ViewerChips<'a> {
    view: &'a ViewerChipsView,
}

impl<'a> ViewerChips<'a> {
    // Paper: the chips sit 14 in from the picture's edge, 8 apart; the warning is 6 under the mode chip
    const INSET: f32 = 14.0;
    const WARNING_GAP: f32 = 6.0;
    // Paper: the owner tag is 6 from the top, 26 high with 14 each side; the chip row then starts at 40
    const OWNER_TOP: f32 = 6.0;
    const OWNER_HEIGHT: f32 = 26.0;
    const OWNER_PADDING: f32 = 14.0;
    const OWNER_SEPARATOR: f32 = 7.0;
    const ROW_WITH_OWNER: f32 = 40.0;
    const OWNER: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();
    // Paper: the readout is 30 high with 14 each side; a label is a space from its value, groups two spaces apart,
    // and the angles and the note three
    const READOUT_HEIGHT: f32 = 30.0;
    const READOUT_PADDING: f32 = 14.0;
    const LABEL_GAP: f32 = 4.0;
    const GROUP_GAP: f32 = 9.0;
    const WIDE_GAP: f32 = 14.0;
    const READOUT: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);
    // Paper `30B-0`: the scene camera frame is 3 px, with 4 px corner brackets 34 long, 7 in
    const FRAME: f32 = 3.0;
    const BRACKET: f32 = 4.0;
    const BRACKET_LENGTH: f32 = 34.0;
    const BRACKET_INSET: f32 = 7.0;

    pub fn new(view: &'a ViewerChipsView) -> Self {
        Self { view }
    }

    pub fn show(&self, ui: &mut Ui, viewer: Rect) {
        self.paint(ui.painter(), viewer);
    }

    pub fn paint(&self, painter: &Painter, viewer: Rect) {
        let mut top = viewer.top() + Self::INSET;
        if let Some(owner) = &self.view.owner {
            if matches!(owner, ViewerOwner::SceneCamera(_)) {
                Self::scene_camera_frame(painter, viewer);
            }
            Self::owner_tag(painter, viewer, owner);
            top = viewer.top() + Self::ROW_WITH_OWNER;
        }
        let gap = <spacing::S as SpacingToken>::VALUE;
        let room = (viewer.width() - Self::INSET * 2.0).max(0.0);
        let mut left_limit = viewer.right() - Self::INSET;
        if !self.view.mode.is_empty() {
            let tone = self.view.tone;
            let pill = ViewerPill { dot: Some(tone.color()), border: tone.border(), parts: &self.view.mode };
            let width = pill.width(painter.ctx()).min(room);
            let rect =
                Rect::from_min_size(pos2(viewer.right() - Self::INSET - width, top), vec2(width, ViewerPill::HEIGHT));
            pill.paint(painter, rect);
            left_limit = rect.left() - gap;
        }
        if let Some(warning) = &self.view.warning {
            let parts = [warning.clone()];
            let pill =
                ViewerPill { dot: Some(<color::Warning as ColorToken>::VALUE), border: Self::overlay(), parts: &parts };
            let width = pill.width(painter.ctx()).min(room);
            let y = top + ViewerPill::HEIGHT + Self::WARNING_GAP;
            pill.paint(
                painter,
                Rect::from_min_size(pos2(viewer.right() - Self::INSET - width, y), vec2(width, ViewerPill::HEIGHT)),
            );
        }
        self.source(painter, pos2(viewer.left() + Self::INSET, top), left_limit);
        if let Some(readout) = &self.view.readout {
            Self::readout(painter, viewer, readout);
        }
    }

    // the source chip, then the lens chip when it fits whole, between `at` and `right`
    fn source(&self, painter: &Painter, at: egui::Pos2, right: f32) {
        let (dot, parts) = match &self.view.source {
            ViewerSource::Scene { name, color, index, count } => {
                let args = Args::new().uint("index", *index as u64).uint("count", *count as u64);
                let place = Text::format(StudioUiMessage::ScenePosition, args);
                (Some(color.color()), vec![Text::message(StudioUiMessage::ViewerScene), name.clone(), place])
            }
            ViewerSource::Original => {
                (None, vec![Text::message(StudioUiMessage::Original), Text::message(StudioUiMessage::AsRecorded)])
            }
        };
        let pill = ViewerPill { dot, border: Self::overlay(), parts: &parts };
        let width = pill.width(painter.ctx()).min((right - at.x).max(0.0));
        if width <= 0.0 {
            return;
        }
        let rect = Rect::from_min_size(at, vec2(width, ViewerPill::HEIGHT));
        pill.paint(painter, rect);
        if let Some(lens) = &self.view.lens {
            let parts = [lens.clone()];
            let pill = ViewerPill { dot: None, border: Self::overlay(), parts: &parts };
            let x = rect.right() + <spacing::S as SpacingToken>::VALUE;
            let width = pill.width(painter.ctx());
            if x + width <= right {
                pill.paint(painter, Rect::from_min_size(pos2(x, at.y), vec2(width, ViewerPill::HEIGHT)));
            }
        }
    }

    // the tag at the top centre: who drives the camera, and the scene or player name
    pub(crate) fn owner_tag(painter: &Painter, viewer: Rect, owner: &ViewerOwner) {
        let (fill, word, name) = match owner {
            ViewerOwner::SceneCamera(name) => {
                (<studio::SceneCamera as ColorToken>::VALUE, StudioUiMessage::SceneCamera, name)
            }
            ViewerOwner::Following(name) => (<color::Accent as ColorToken>::VALUE, StudioUiMessage::Following, name),
        };
        let ctx = painter.ctx();
        let ink = <color::BgApp as ColorToken>::VALUE;
        let word = UiText::resolve(ctx, &Text::message(word));
        let name = UiText::resolve(ctx, name);
        let word_width = Draw::measure(ctx, &word, Self::OWNER).x;
        let name_width = Draw::measure(ctx, &name, Self::OWNER).x;
        let full = Self::OWNER_PADDING * 2.0 + word_width + Self::OWNER_SEPARATOR * 2.0 + name_width;
        let width = full.min(viewer.width() - Self::INSET * 2.0).max(0.0);
        let tag = Rect::from_min_size(
            pos2(viewer.center().x - width / 2.0, viewer.top() + Self::OWNER_TOP),
            vec2(width, Self::OWNER_HEIGHT),
        );
        Draw::fill(painter, tag, Corner::Small, fill);
        let right = tag.right() - Self::OWNER_PADDING;
        let x = tag.left() + Self::OWNER_PADDING;
        let galley = Draw::galley(painter, &word, Self::OWNER, ink, TextFit::Truncate((right - x).max(0.0)));
        let at = Align2::LEFT_CENTER.anchor_size(pos2(x, tag.center().y), galley.size());
        painter.galley(at.min, galley, ink);
        let x = at.right() + Self::OWNER_SEPARATOR * 2.0;
        if x < right {
            painter.circle_filled(pos2(at.right() + Self::OWNER_SEPARATOR, tag.center().y), 1.0, ink);
            let galley = Draw::galley(painter, &name, Self::OWNER, ink, TextFit::Truncate(right - x));
            let at = Align2::LEFT_CENTER.anchor_size(pos2(x, tag.center().y), galley.size());
            painter.galley(at.min, galley, ink);
        }
    }

    // the 3 px frame round the picture and the four corner brackets
    fn scene_camera_frame(painter: &Painter, viewer: Rect) {
        let hue = <studio::SceneCamera as ColorToken>::VALUE;
        painter.rect_stroke(viewer, Corner::Small.radius(viewer), Stroke::new(Self::FRAME, hue), StrokeKind::Inside);
        let inner = viewer.shrink(Self::BRACKET_INSET);
        let long = Self::BRACKET_LENGTH;
        let stroke = Stroke::new(Self::BRACKET, hue);
        for (corner, x, y) in [
            (inner.left_top(), 1.0, 1.0),
            (inner.right_top(), -1.0, 1.0),
            (inner.left_bottom(), 1.0, -1.0),
            (inner.right_bottom(), -1.0, -1.0),
        ] {
            let points = vec![corner + vec2(0.0, y * long), corner, corner + vec2(x * long, 0.0)];
            painter.add(egui::Shape::line(points, stroke));
        }
    }

    // the camera readout at the bottom left: as many runs as the layout and the picture's width allow
    pub(crate) fn readout(painter: &Painter, viewer: Rect, readout: &CameraReadout) {
        let layout = ReadoutLayout::for_width(viewer.width());
        let dim = <color::TextDim as ColorToken>::VALUE;
        let ink = <color::Text as ColorToken>::VALUE;
        let message = |word| Text::message(word);
        let mut runs: Vec<(Text, Color32, f32)> = vec![
            (message(StudioUiMessage::AxisX), <color::AxisX as ColorToken>::VALUE, 0.0),
            (readout.x.clone(), ink, Self::LABEL_GAP),
            (message(StudioUiMessage::AxisY), <color::AxisY as ColorToken>::VALUE, Self::GROUP_GAP),
            (readout.y.clone(), ink, Self::LABEL_GAP),
            (message(StudioUiMessage::AxisZ), <color::AxisZ as ColorToken>::VALUE, Self::GROUP_GAP),
            (readout.z.clone(), ink, Self::LABEL_GAP),
        ];
        if layout != ReadoutLayout::Small {
            runs.extend([
                (message(StudioUiMessage::Yaw), dim, Self::WIDE_GAP),
                (readout.yaw.clone(), ink, Self::LABEL_GAP),
                (message(StudioUiMessage::Pitch), dim, Self::GROUP_GAP),
                (readout.pitch.clone(), ink, Self::LABEL_GAP),
                (message(StudioUiMessage::Roll), dim, Self::GROUP_GAP),
                (readout.roll.clone(), ink, Self::LABEL_GAP),
            ]);
        }
        if layout == ReadoutLayout::Wide
            && let Some(extra) = &readout.extra
        {
            runs.push((extra.clone(), dim, Self::WIDE_GAP));
        }
        let ctx = painter.ctx();
        let resolved: Vec<(String, Color32, f32)> =
            runs.into_iter().map(|(text, hue, gap)| (UiText::resolve(ctx, &text), hue, gap)).collect();
        let room = (viewer.width() - Self::INSET * 2.0 - Self::READOUT_PADDING * 2.0).max(0.0);
        let mut width = 0.0;
        let mut fitted = Vec::new();
        for (text, hue, gap) in &resolved {
            let next = width + gap + Draw::measure(ctx, text, Self::READOUT).x;
            if next > room {
                break;
            }
            width = next;
            fitted.push((text, *hue, *gap));
        }
        if fitted.is_empty() {
            return;
        }
        let pill = Rect::from_min_size(
            pos2(viewer.left() + Self::INSET, viewer.bottom() - Self::INSET - Self::READOUT_HEIGHT),
            vec2(width + Self::READOUT_PADDING * 2.0, Self::READOUT_HEIGHT),
        );
        Draw::fill(painter, pill, Corner::Full, <color::Pill as ColorToken>::VALUE);
        Draw::stroke(painter, pill, Corner::Full, Self::overlay());
        let mut x = pill.left() + Self::READOUT_PADDING;
        for (text, hue, gap) in fitted {
            x += gap;
            let galley = Draw::galley(painter, text, Self::READOUT, hue, TextFit::Line);
            let at = Align2::LEFT_CENTER.anchor_size(pos2(x, pill.center().y), galley.size());
            x = at.right();
            painter.galley(at.min, galley, hue);
        }
    }

    fn overlay() -> Color32 {
        <color::LineOverlay as ColorToken>::VALUE
    }
}
