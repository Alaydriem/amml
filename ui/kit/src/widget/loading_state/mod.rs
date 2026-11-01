mod loading_kind;

pub use loading_kind::LoadingKind;

use egui::{Painter, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::glyph::{Spinner, SpinnerSize};
use crate::motion::Motion;
use crate::text::UiText;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Loading state`. Static skeleton rows for a list, or an inline
/// spinner with a label. The skeleton does not shimmer.
pub struct LoadingState {
    kind: LoadingKind,
}

impl LoadingState {
    const LABEL: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);
    /// Paper: Loading state, the 16:9 thumbnail block.
    const THUMB_WIDTH: f32 = 80.0;
    const THUMB_HEIGHT: f32 = 45.0;
    /// Paper: Loading state, the title bar and the sub bar under it. Their widths go round these
    /// lists, row by row, so the rows do not look alike.
    const TITLE_HEIGHT: f32 = 12.0;
    const TITLE_WIDTHS: [f32; 4] = [220.0, 180.0, 250.0, 160.0];
    const SUB_HEIGHT: f32 = 10.0;
    const SUB_WIDTHS: [f32; 4] = [140.0, 120.0, 140.0, 120.0];
    /// Paper: Loading state, the trailing bar.
    const TRAIL_WIDTH: f32 = 48.0;
    const TRAIL_HEIGHT: f32 = 10.0;

    pub fn new(kind: LoadingKind) -> Self {
        Self { kind }
    }

    /// Takes the full width: one `--spacing-lane` per skeleton row, or one `--spacing-button` line.
    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), self.height()), Sense::hover());
        self.paint(ui.painter(), rect);
        response
    }

    pub fn height(&self) -> f32 {
        match &self.kind {
            LoadingKind::Skeleton { rows } => <spacing::Lane as SpacingToken>::VALUE * *rows as f32,
            LoadingKind::Inline(_) => <spacing::Button as SpacingToken>::VALUE,
        }
    }

    /// Paints into `rect`. The inline spinner turns with the context's time.
    pub fn paint(&self, painter: &Painter, rect: Rect) {
        match &self.kind {
            LoadingKind::Skeleton { rows } => {
                let lane = <spacing::Lane as SpacingToken>::VALUE;
                for index in 0..*rows {
                    let row = Rect::from_min_size(
                        pos2(rect.left(), rect.top() + lane * index as f32),
                        vec2(rect.width(), lane),
                    );
                    Self::paint_skeleton_row(painter, row, index);
                }
            }
            LoadingKind::Inline(label) => {
                let padding = <spacing::S as SpacingToken>::VALUE;
                let side = SpinnerSize::Small.side();
                let spinner =
                    Rect::from_center_size(pos2(rect.left() + padding + side / 2.0, rect.center().y), vec2(side, side));
                let turn = Motion::turn(painter.ctx());
                Spinner { size: SpinnerSize::Small, turn }.paint(painter, spinner, <color::Text as ColorToken>::VALUE);
                let left = spinner.right() + <spacing::S as SpacingToken>::VALUE;
                let colour = <color::TextDim as ColorToken>::VALUE;
                let text = UiText::resolve(painter.ctx(), label);
                let room = (rect.right() - padding - left).max(0.0);
                let galley = Draw::galley(painter, &text, Self::LABEL, colour, TextFit::Truncate(room));
                painter.galley(pos2(left, rect.center().y - galley.size().y / 2.0), galley, colour);
            }
        }
    }

    // the thumbnail, the title and sub bars, and the trailing bar, `--spacing-s` in from each side
    fn paint_skeleton_row(painter: &Painter, row: Rect, index: usize) {
        let padding = <spacing::S as SpacingToken>::VALUE;
        let gap = <spacing::M as SpacingToken>::VALUE;
        let centre = row.center().y;
        let block = |rect: Rect| Draw::fill(painter, rect, Corner::Small, <color::BgRaised as ColorToken>::VALUE);
        let thumb = Rect::from_min_size(
            pos2(row.left() + padding, centre - Self::THUMB_HEIGHT / 2.0),
            vec2(Self::THUMB_WIDTH, Self::THUMB_HEIGHT),
        );
        block(thumb);
        let trail = Rect::from_min_size(
            pos2(row.right() - padding - Self::TRAIL_WIDTH, centre - Self::TRAIL_HEIGHT / 2.0),
            vec2(Self::TRAIL_WIDTH, Self::TRAIL_HEIGHT),
        );
        block(trail);
        // the bars stay inside the space between the thumbnail and the trailing bar
        let left = thumb.right() + gap;
        let room = (trail.left() - gap - left).max(0.0);
        let bars = Self::TITLE_HEIGHT + padding + Self::SUB_HEIGHT;
        let top = centre - bars / 2.0;
        let title = Self::TITLE_WIDTHS[index % Self::TITLE_WIDTHS.len()].min(room);
        block(Rect::from_min_size(pos2(left, top), vec2(title, Self::TITLE_HEIGHT)));
        let sub = Self::SUB_WIDTHS[index % Self::SUB_WIDTHS.len()].min(room);
        block(Rect::from_min_size(pos2(left, top + Self::TITLE_HEIGHT + padding), vec2(sub, Self::SUB_HEIGHT)));
    }
}
