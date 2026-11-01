use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::glyph::{Spinner, SpinnerSize};
use amml_ui::motion::Motion;
use amml_ui::tokens::{ColorToken, color};
use egui::{Align2, Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

use crate::message::StudioUiMessage;

/// Paper: Editor components, `Viewer chips and overlays` (`3TE-0`) section D, and `Viewer, original, typing`
/// (`3I7-0`). What the viewer shows over, or in place of, the game picture: the rebuild card while the world is
/// rebuilt from a checkpoint, the join card while the replay server starts, and the empty hatched surface while the
/// player types in the game.
#[derive(Debug, Clone)]
pub enum ViewerOverlay {
    /// `progress` runs from 0 to 1; any other value is clamped, and not a number shows as 0. `detail` is the line
    /// under the title, as in "from checkpoint 00:00:10".
    Rebuilding {
        progress: f32,
        detail: Text,
    },
    /// `title` as in "Joining “Nether trip”", `body` the attempt line and `hint` the faint line under it.
    Joining {
        title: Text,
        body: Text,
        hint: Text,
    },
    Typing,
}

impl ViewerOverlay {
    // Paper `3TE-0`: the rebuild card is 400 × 96; text 50 in, the bar 16 in at 72 and 6 high
    const REBUILD: egui::Vec2 = vec2(400.0, 96.0);
    const REBUILD_TEXT: f32 = 50.0;
    const REBUILD_SPINNER: Pos2 = pos2(28.0, 30.0);
    const BAR_INSET: f32 = 16.0;
    const BAR_TOP: f32 = 72.0;
    const BAR_HEIGHT: f32 = 6.0;
    // Paper: the rebuild card's border is `--color-warning` at 60 %
    const REBUILD_BORDER_ALPHA: u8 = 0x99;
    // Paper `3TE-0`: the join card is 480 × 120; text 56 in at 25, 55 and 80
    const JOIN: egui::Vec2 = vec2(480.0, 120.0);
    const JOIN_TEXT: f32 = 56.0;
    const JOIN_SPINNER: Pos2 = pos2(32.0, 36.0);
    const JOIN_LINES: [f32; 3] = [25.0, 55.0, 80.0];
    // cards keep this far from the viewer's edge
    const MARGIN: f32 = 16.0;
    // Paper `3I7-0`: 12 px stripes 12 apart at 45 degrees, and a dashed `--color-line-strong` border
    const STRIPE: f32 = 12.0;
    const DASH: f32 = 4.0;

    pub fn paint(&self, painter: &Painter, viewer: Rect) {
        match self {
            Self::Rebuilding { progress, detail } => Self::rebuilding(painter, viewer, *progress, detail),
            Self::Joining { title, body, hint } => Self::joining(painter, viewer, title, body, hint),
            Self::Typing => Self::typing(painter, viewer),
        }
    }

    fn rebuilding(painter: &Painter, viewer: Rect, progress: f32, detail: &Text) {
        let card = Self::card(viewer, Self::REBUILD);
        let warning = <color::Warning as ColorToken>::VALUE;
        Draw::fill(painter, card, Corner::Panel, <color::Card as ColorToken>::VALUE);
        Draw::stroke(painter, card, Corner::Panel, warning.gamma_multiply_u8(Self::REBUILD_BORDER_ALPHA));
        Self::spinner(painter, card.min + Self::REBUILD_SPINNER.to_vec2(), warning);
        let lines = [
            (
                Text::message(StudioUiMessage::RebuildingWorld),
                TextStyle::new(TypeSize::Label, FontFace::Semibold),
                <color::Text as ColorToken>::VALUE,
                20.0,
            ),
            (
                detail.clone(),
                TextStyle::new(TypeSize::Caption, FontFace::Regular),
                <color::TextDim as ColorToken>::VALUE,
                44.0,
            ),
        ];
        Self::lines(painter, card, Self::REBUILD_TEXT, &lines);
        let track = Rect::from_min_size(
            card.min + vec2(Self::BAR_INSET, Self::BAR_TOP),
            vec2(card.width() - Self::BAR_INSET * 2.0, Self::BAR_HEIGHT),
        );
        Draw::fill(painter, track, Corner::Full, <color::BgField as ColorToken>::VALUE);
        let done = if progress.is_nan() { 0.0 } else { progress.clamp(0.0, 1.0) };
        if done > 0.0 {
            let fill = Rect::from_min_size(track.min, vec2(track.width() * done, track.height()));
            Draw::fill(painter, fill, Corner::Full, warning);
        }
    }

    fn joining(painter: &Painter, viewer: Rect, title: &Text, body: &Text, hint: &Text) {
        let card = Self::card(viewer, Self::JOIN);
        Draw::fill(painter, card, Corner::Panel, <color::BgRaised as ColorToken>::VALUE);
        Draw::stroke(painter, card, Corner::Panel, <color::LineStrong as ColorToken>::VALUE);
        Self::spinner(painter, card.min + Self::JOIN_SPINNER.to_vec2(), <color::Accent as ColorToken>::VALUE);
        let [first, second, third] = Self::JOIN_LINES;
        let lines = [
            (
                title.clone(),
                TextStyle::new(TypeSize::Body, FontFace::Semibold),
                <color::Text as ColorToken>::VALUE,
                first,
            ),
            (
                body.clone(),
                TextStyle::new(TypeSize::Note, FontFace::Regular),
                <color::TextDim as ColorToken>::VALUE,
                second,
            ),
            (
                hint.clone(),
                TextStyle::new(TypeSize::Caption, FontFace::Regular),
                <color::TextFaint as ColorToken>::VALUE,
                third,
            ),
        ];
        Self::lines(painter, card, Self::JOIN_TEXT, &lines);
    }

    fn typing(painter: &Painter, viewer: Rect) {
        let stripe = <color::LineOverlay as ColorToken>::VALUE;
        // a stripe runs from top left to bottom right; `offset` moves it across the viewer
        let reach = Self::STRIPE * std::f32::consts::SQRT_2;
        let long = viewer.width() + viewer.height();
        let mut offset = -viewer.width();
        while offset < viewer.height() {
            let band = [
                pos2(viewer.left() - long, viewer.top() + offset - long),
                pos2(viewer.right() + long, viewer.top() + offset + viewer.width() + long),
                pos2(viewer.right() + long, viewer.top() + offset + reach + viewer.width() + long),
                pos2(viewer.left() - long, viewer.top() + offset + reach - long),
            ];
            let clipped = Self::clip(band.to_vec(), viewer);
            if clipped.len() >= 3 {
                painter.add(Shape::convex_polygon(clipped, stripe, Stroke::NONE));
            }
            offset += reach * 2.0;
        }
        let edge = viewer.shrink(0.5);
        let frame = [edge.left_top(), edge.right_top(), edge.right_bottom(), edge.left_bottom(), edge.left_top()];
        let border = Stroke::new(1.0, <color::LineStrong as ColorToken>::VALUE);
        painter.extend(Shape::dashed_line(&frame, border, Self::DASH, Self::DASH));
    }

    // a card of `size` centred in the viewer, no wider or taller than the viewer less the margin
    fn card(viewer: Rect, size: egui::Vec2) -> Rect {
        let room = (viewer.size() - vec2(Self::MARGIN, Self::MARGIN) * 2.0).max(egui::Vec2::ZERO);
        Rect::from_center_size(viewer.center(), size.min(room))
    }

    fn spinner(painter: &Painter, centre: Pos2, hue: Color32) {
        let side = SpinnerSize::Regular.side();
        let turn = Motion::turn(painter.ctx());
        Spinner { size: SpinnerSize::Regular, turn }.paint(
            painter,
            Rect::from_center_size(centre, vec2(side, side)),
            hue,
        );
    }

    // each line at its top, `left` into the card, cut at the card's right padding; lines below the card are left out
    fn lines(painter: &Painter, card: Rect, left: f32, lines: &[(Text, TextStyle, Color32, f32)]) {
        let x = card.left() + left;
        let room = (card.right() - Self::MARGIN - x).max(0.0);
        for (text, style, hue, top) in lines {
            let y = card.top() + top;
            if room <= 0.0 || y + style.size.line_height() > card.bottom() {
                continue;
            }
            let resolved = amml_ui::text::UiText::resolve(painter.ctx(), text);
            let galley = Draw::galley(painter, &resolved, *style, *hue, TextFit::Truncate(room));
            let at = Align2::LEFT_TOP.anchor_size(pos2(x, y), galley.size());
            painter.galley(at.min, galley, *hue);
        }
    }

    // the part of the convex polygon `points` inside `rect` (Sutherland–Hodgman, one rect edge at a time)
    fn clip(points: Vec<Pos2>, rect: Rect) -> Vec<Pos2> {
        // each rect edge: the axis it cuts (0 is x, 1 is y), where, and whether the inside is above that value
        let edges =
            [(0, rect.left(), true), (0, rect.right(), false), (1, rect.top(), true), (1, rect.bottom(), false)];
        let mut out = points;
        for (axis, bound, above) in edges {
            let inside = |point: Pos2| if above { point[axis] >= bound } else { point[axis] <= bound };
            let input = std::mem::take(&mut out);
            for (index, &current) in input.iter().enumerate() {
                let previous = input[(index + input.len() - 1) % input.len()];
                match (inside(previous), inside(current)) {
                    (true, true) => out.push(current),
                    (true, false) => out.push(Self::cross(previous, current, axis, bound)),
                    (false, true) => {
                        out.push(Self::cross(previous, current, axis, bound));
                        out.push(current);
                    }
                    (false, false) => {}
                }
            }
            if out.is_empty() {
                break;
            }
        }
        out
    }

    // where the line from `a` to `b` meets `axis` = `bound`
    fn cross(a: Pos2, b: Pos2, axis: usize, bound: f32) -> Pos2 {
        let t = (bound - a[axis]) / (b[axis] - a[axis]);
        let mut point = a + (b - a) * t;
        point[axis] = bound;
        point
    }
}
