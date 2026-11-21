use amml_abi::{AmmlCameraKind, AmmlCameraTable, AmmlPose, AmmlStr};

use crate::{ApiError, CameraKind, Client, Pose};

/// The client's camera outside a `CameraSetup` frame (`Ctx::camera`): the game's own free camera.
///
/// `set_free` asks the client to show the world from a pose through the game's `minecraft:free` camera preset, the way
/// a server's camera command does; the camera stays there until `release_free`, a server camera instruction, a world
/// leave or this mod's disable (the loader then sends the clear). One mod holds the free camera at a time. Game thread
/// only. `Area::CAMERA_FREE`.
///
/// `active_camera` reads which vanilla camera the client shows the world through (`Area::CAMERA_ACTIVE`).
pub struct CameraControl {
    table: Option<AmmlCameraTable>,
    module: &'static str,
}

impl CameraControl {
    pub(crate) fn new(table: Option<AmmlCameraTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// Shows the world from `pose` (position, yaw and pitch; the roll is not used). A call again moves the camera.
    /// `Err(NotReady)` while no world is joined, and before the loader saw the client's packet handler in this world
    /// (at the next server packet of a hooked handler slot, such as the chunk publisher update); `Err(NotAvailable)`
    /// when the world has no `minecraft:free` preset or without the area; `Err(Refused)` off the game thread, from a
    /// listener of the `CameraInstructionReceived` of a free camera call (origin `THIS_MOD` or `OTHER_MOD`), or when
    /// another mod holds the free camera; `Err(Loader(INVALID_ARGUMENT))` for a number that is not finite. Your
    /// listeners of `CameraInstructionReceived` see the instruction with origin `THIS_MOD`.
    #[allow(unsafe_code)]
    pub fn set_free(&self, pose: Pose) -> Result<(), ApiError> {
        let Some(set_free) = self.table.and_then(|table| table.set_free) else { return Err(ApiError::NotAvailable) };
        let pose = AmmlPose::from(pose);
        // SAFETY: the table function is the loader's; the module id and the pose live for the call.
        let status = unsafe { set_free(AmmlStr::from_str(self.module), &raw const pose) };
        ApiError::check(status)
    }

    /// Gives the camera back to the player and stops holding the free camera. `Ok` with nothing sent when no mod holds
    /// it; `Err(Refused)` when another mod holds it. The other errors are those of `set_free`.
    #[allow(unsafe_code)]
    pub fn release_free(&self) -> Result<(), ApiError> {
        let Some(release_free) = self.table.and_then(|table| table.release_free) else {
            return Err(ApiError::NotAvailable);
        };
        // SAFETY: the table function is the loader's; the module id lives for the call.
        let status = unsafe { release_free(AmmlStr::from_str(self.module)) };
        ApiError::check(status)
    }

    /// Whether the joined world has the `minecraft:free` preset, so `set_free` can work. `None` while no world is
    /// joined, off the game thread, or without the area.
    #[allow(unsafe_code)]
    pub fn free_available(&self) -> Option<bool> {
        let free_available = self.table.and_then(|table| table.free_available)?;
        let mut out = 0u8;
        // SAFETY: the table function is the loader's; `out` is writable for the call.
        let status = unsafe { free_available(&raw mut out) };
        status.is_ok().then_some(out != 0)
    }

    /// The vanilla camera `client` shows the world through: first person, third person, third person from the front, or
    /// `Other` for any other camera (a free camera, a camera a server set). Game thread only, for the client of the
    /// event that gave it to you. `None` off the game thread, while no world is joined, when the game has no active
    /// camera, without `Area::CAMERA_ACTIVE`, and from a loader before camera table v3.
    #[allow(unsafe_code)]
    pub fn active_camera(&self, client: &Client<'_>) -> Option<CameraKind> {
        let active_camera = self.table.and_then(|table| table.active_camera)?;
        let mut out = AmmlCameraKind::OTHER;
        // SAFETY: the table function is the loader's, which checks the handle before it reads game memory; `out` is a
        // live local.
        let status = unsafe { active_camera(client.handle(), &raw mut out) };
        status.is_ok().then(|| CameraKind::from_abi(out))
    }
}
