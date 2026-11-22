use amml_abi::AmmlBlockEntityCamera;

/// The camera of a `BlockEntityPass`: the local player's level renderer camera (the view the player sees), or any other
/// level renderer camera.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockEntityCamera {
    Player,
    Other,
}

impl From<AmmlBlockEntityCamera> for BlockEntityCamera {
    fn from(camera: AmmlBlockEntityCamera) -> Self {
        if camera == AmmlBlockEntityCamera::PLAYER { Self::Player } else { Self::Other }
    }
}

impl From<BlockEntityCamera> for AmmlBlockEntityCamera {
    fn from(camera: BlockEntityCamera) -> Self {
        match camera {
            BlockEntityCamera::Player => Self::PLAYER,
            BlockEntityCamera::Other => Self::OTHER,
        }
    }
}
