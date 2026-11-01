mod path_map_action;
mod path_map_camera;
mod path_map_point;
mod path_map_view;

pub use path_map_action::PathMapAction;
pub use path_map_camera::PathMapCamera;
pub use path_map_point::PathMapPoint;
pub use path_map_view::PathMapView;

use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::glyph::Glyph;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Align2, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2, pos2, vec2};

use crate::glyph::PathMapCone;
use crate::message::StudioUiMessage;
use crate::tokens::color as studio;

/// Paper: Editor components, `Path map` (`46E-0`). The scene's camera path seen from above: the keys on their
/// path, the players, and the camera cone, fitted into a small map. A click on a key selects it.
pub struct PathMap<'a> {
    view: &'a PathMapView,
}

impl<'a> PathMap<'a> {
    /// Paper: the map is 250 × 170.
    pub const SIZE: Vec2 = vec2(250.0, 170.0);
    // Paper: grid lines every 30, the first at x 35 and y 45; the verticals start under the label at 20
    const GRID_STEP: f32 = 30.0;
    const GRID_START: Pos2 = pos2(35.0, 45.0);
    const GRID_TOP: f32 = 20.0;
    // Paper: the label is 10 in and 5 down
    const LABEL: Pos2 = pos2(10.0, 5.0);
    // the points are fitted inside the label and far enough in that the camera cone stays on the map; a path
    // shorter than this many blocks is not blown up to fill the map
    const LABEL_BAND: f32 = 24.0;
    const MIN_SPAN: f32 = 32.0;
    // Paper: the path is the scene colour at 90 %; keys are 4 px half diamonds, the selected key 6 with a radius 9
    // ring; players are radius 3.5 dots
    const PATH_ALPHA: u8 = 0xE6;
    const KEY_HALF: f32 = 4.0;
    const SELECTED_HALF: f32 = 6.0;
    const SELECTED_RING: f32 = 9.0;
    const PLAYER_RADIUS: f32 = 3.5;
    // a click this close to a key selects it
    const HIT: f32 = 8.0;

    pub fn new(view: &'a PathMapView) -> Self {
        Self { view }
    }

    pub fn show(&self, ui: &mut Ui) -> Option<PathMapAction> {
        let (rect, response) = ui.allocate_exact_size(Self::SIZE, Sense::click());
        self.paint(ui.painter(), rect);
        let at = response.interact_pointer_pos().filter(|_| response.clicked())?;
        self.waypoints(rect)
            .into_iter()
            .enumerate()
            .map(|(index, point)| (index, point.distance(at)))
            .filter(|(_, distance)| *distance <= Self::HIT)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| PathMapAction::SelectKey(index))
    }

    /// Where each key of the path is drawn on a map at `rect`. None while the track is hidden.
    pub fn waypoints(&self, rect: Rect) -> Vec<Pos2> {
        if self.view.hidden {
            return Vec::new();
        }
        self.view.path.iter().map(|point| self.place(rect, point.at)).collect()
    }

    pub fn paint(&self, painter: &Painter, rect: Rect) {
        Draw::fill(painter, rect, Corner::Panel, <color::BgOverlay as ColorToken>::VALUE);
        Self::grid(painter, rect);
        let label = TextStyle::new(TypeSize::Micro, FontFace::Semibold);
        let dim = <color::TextDim as ColorToken>::VALUE;
        let room = (rect.width() - Self::LABEL.x * 2.0).max(0.0);
        Draw::text(
            painter,
            rect.min + Self::LABEL.to_vec2(),
            Align2::LEFT_TOP,
            &self.view.label,
            label,
            dim,
            TextFit::Truncate(room),
        );
        if self.view.hidden {
            let note = Text::message(StudioUiMessage::CameraTrackHidden);
            let style = TextStyle::new(TypeSize::Caption, FontFace::Regular);
            let faint = <color::TextFaint as ColorToken>::VALUE;
            Draw::text(painter, rect.center(), Align2::CENTER_CENTER, &note, style, faint, TextFit::Truncate(room));
        } else {
            self.path(painter, rect);
        }
        let camera = if self.view.scene_drives {
            <studio::SceneCamera as ColorToken>::VALUE
        } else {
            <color::Playhead as ColorToken>::VALUE
        };
        let eye = self.place(rect, self.view.camera.at);
        let cone = PathMapCone { heading: self.view.camera.heading };
        cone.paint(painter, Rect::from_center_size(eye, Vec2::splat(PathMapCone::REACH * 2.0)), camera);
        for player in &self.view.players {
            painter.circle_filled(self.place(rect, *player), Self::PLAYER_RADIUS, dim);
        }
        Draw::stroke(painter, rect, Corner::Panel, <color::LineOverlay as ColorToken>::VALUE);
    }

    fn grid(painter: &Painter, rect: Rect) {
        let stroke = Stroke::new(1.0, <color::LineOverlay as ColorToken>::VALUE);
        let mut x = rect.left() + Self::GRID_START.x;
        while x < rect.right() {
            painter.line_segment([pos2(x + 0.5, rect.top() + Self::GRID_TOP), pos2(x + 0.5, rect.bottom())], stroke);
            x += Self::GRID_STEP;
        }
        let mut y = rect.top() + Self::GRID_START.y;
        while y < rect.bottom() {
            painter.line_segment([pos2(rect.left(), y + 0.5), pos2(rect.right(), y + 0.5)], stroke);
            y += Self::GRID_STEP;
        }
    }

    // the path line, then a diamond on each key
    fn path(&self, painter: &Painter, rect: Rect) {
        let points = self.waypoints(rect);
        if points.len() > 1 {
            let hue = self.view.scene.color().gamma_multiply_u8(Self::PATH_ALPHA);
            painter.add(Shape::line(points.clone(), Stroke::new(<spacing::StrokeCurve as SpacingToken>::VALUE, hue)));
        }
        let edge = Stroke::new(1.0, <color::BgApp as ColorToken>::VALUE);
        for (point, at) in self.view.path.iter().zip(points) {
            if point.selected {
                let white = <color::OnAccent as ColorToken>::VALUE;
                painter.add(Shape::convex_polygon(Self::diamond(at, Self::SELECTED_HALF), white, edge));
                painter.circle_stroke(
                    at,
                    Self::SELECTED_RING,
                    Stroke::new(<spacing::StrokeThin as SpacingToken>::VALUE, white),
                );
            } else {
                let fill = <studio::SceneCamera as ColorToken>::VALUE;
                painter.add(Shape::convex_polygon(Self::diamond(at, Self::KEY_HALF), fill, edge));
            }
        }
    }

    fn diamond(centre: Pos2, half: f32) -> Vec<Pos2> {
        vec![centre - vec2(0.0, half), centre + vec2(half, 0.0), centre + vec2(0.0, half), centre - vec2(half, 0.0)]
    }

    // a world position on a map at `rect`: every point the map shows, fitted into the room under the label, the
    // same scale on both axes
    fn place(&self, rect: Rect, world: Pos2) -> Pos2 {
        let reach = PathMapCone::REACH;
        let room = Rect::from_min_max(
            pos2(rect.left() + reach, rect.top() + reach.max(Self::LABEL_BAND)),
            pos2(rect.right() - reach, rect.bottom() - reach),
        );
        let all = self.view.path.iter().map(|point| point.at).chain(self.view.players.iter().copied());
        let bounds = all.fold(Rect::from_min_max(self.view.camera.at, self.view.camera.at), |bounds, point| {
            bounds.union(Rect::from_min_max(point, point))
        });
        let span = bounds.size().max(Vec2::splat(Self::MIN_SPAN));
        let scale = (room.width() / span.x).min(room.height() / span.y);
        room.center() + (world - bounds.center()) * scale
    }
}
