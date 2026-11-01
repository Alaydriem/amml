use amml_i18n::text::Text;

/// Paper: `3TE-0` section B, the owner tags. Who drives the camera when it is not the free camera: the scene camera
/// of the named scene, or the camera that follows the named player.
#[derive(Debug, Clone)]
pub enum ViewerOwner {
    SceneCamera(Text),
    Following(Text),
}
