use amml_i18n::text::Text;
use egui::Pos2;

use super::{PathMapCamera, PathMapPoint};
use crate::tokens::selector::SceneColor;

/// What the path map shows. Positions are world positions seen from above; the map fits them in.
#[derive(Debug, Clone)]
pub struct PathMapView {
    /// The label at the top left, as in "Path · Arrival".
    pub label: Text,
    /// The colour of the scene whose camera path this is.
    pub scene: SceneColor,
    /// The camera keys in time order. Empty for the Original replay.
    pub path: Vec<PathMapPoint>,
    /// The camera track is hidden: no path, and the hidden note instead.
    pub hidden: bool,
    pub camera: PathMapCamera,
    /// The scene camera drives, so the camera cone takes `--color-scene-camera`, not `--color-playhead`.
    pub scene_drives: bool,
    pub players: Vec<Pos2>,
}
