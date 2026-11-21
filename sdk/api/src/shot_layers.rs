use amml_abi::AmmlShotLayers;

/// What a screenshot or a capture frame holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotLayers {
    /// The game image, copied before the first `FramePresent` listener.
    GameOnly,
    /// The image with what the `FramePresent` listeners up to the `NORMAL` band drew.
    WithOverlays,
}

impl From<ShotLayers> for AmmlShotLayers {
    fn from(layers: ShotLayers) -> Self {
        match layers {
            ShotLayers::GameOnly => Self::GAME_ONLY,
            ShotLayers::WithOverlays => Self::WITH_OVERLAYS,
        }
    }
}
