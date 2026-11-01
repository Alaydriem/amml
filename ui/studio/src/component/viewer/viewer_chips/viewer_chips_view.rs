use amml_i18n::text::Text;

use super::{CameraReadout, ViewerOwner, ViewerSource, ViewerTone};

/// What the viewer chips show. Every word except the fixed labels comes from the caller.
#[derive(Debug, Clone)]
pub struct ViewerChipsView {
    /// The top-left chip: the scene that drives the camera, or the Original replay.
    pub source: ViewerSource,
    /// The lens chip after it, as in "FOV 79° · 22 mm".
    pub lens: Option<Text>,
    /// The mode chip's dot and border colour.
    pub tone: ViewerTone,
    /// The mode chip's words, set apart by middle dots, as in "Edit", "Free camera". No chip when empty.
    pub mode: Vec<Text>,
    /// The tag at the top centre while the scene camera drives or the camera follows a player.
    pub owner: Option<ViewerOwner>,
    /// A warning chip under the mode chip.
    pub warning: Option<Text>,
    /// The camera readout at the bottom left.
    pub readout: Option<CameraReadout>,
}
