use core::ffi::c_void;

use crate::{AmmlCameraKind, AmmlPose, AmmlProjection, AmmlStatus, AmmlStr, AmmlVec3};

/// Sets the camera pose of frame `frame` for the mod `module`. `frame` is the `frame_id` of the `CameraSetup` event
/// being dispatched on the calling thread (`REFUSED` for any other value, and for a call outside the dispatch). The
/// pose is for that frame only: the loader writes the rotation after `setupCamera`, the view position after
/// `createViewRenderObject` and the level-camera position after `preRenderUpdate`. `INVALID_ARGUMENT` for a null or
/// non-finite pose, `REFUSED` when another mod set the pose of the frame first. Area `camera.pose`.
pub type CameraSetPoseFn = unsafe extern "C" fn(module: AmmlStr, frame: u64, pose: *const AmmlPose) -> AmmlStatus;

/// Gives in `out` the camera position the loader wrote in the camera frame before frame `frame`. `REFUSED` outside the
/// `CameraSetup` dispatch of `frame` on the calling thread, `FAILED` when the loader wrote no position in that frame.
/// Area `camera.pose`.
pub type CameraRenderedPositionFn = unsafe extern "C" fn(frame: u64, out: *mut AmmlVec3) -> AmmlStatus;

/// Gives in `out` the projection of the camera of frame `frame`, read from the top of its projection stack. Only in
/// the After dispatch of `frame` on the calling thread (`REFUSED` in Before and outside the dispatch); `FAILED` when
/// the matrix holds no usable focal lengths. Area `camera.projection`.
pub type CameraProjectionFn = unsafe extern "C" fn(frame: u64, out: *mut AmmlProjection) -> AmmlStatus;

/// Switches the client's camera to the game's `minecraft:free` preset at `pose` for the mod `module`: the loader makes
/// a camera instruction and hands it to the client's own network handler, as if the server sent it. The camera stays
/// there until `release_free`, a server instruction, a world leave or the mod's disable (the loader then sends the
/// clear). A call again moves the camera. Game thread only (`REFUSED` elsewhere, from a `CameraSetup` listener off the
/// game thread, and from a listener of the `CameraInstructionReceived` of a free camera call); `NOT_READY` while no
/// world is joined and before the loader saw a call of the client's handler in this world; `NOT_AVAILABLE` when the
/// level has no `minecraft:free` preset or the area is not offered; `REFUSED` when another mod holds the free camera;
/// `INVALID_ARGUMENT` for a null or non-finite pose. Raises `CameraInstructionReceived` with origin `THIS_MOD` to the
/// mod's own listeners before it returns. Area `camera.free`.
pub type CameraSetFreeFn = unsafe extern "C" fn(module: AmmlStr, pose: *const AmmlPose) -> AmmlStatus;

/// Sends the clear instruction (the camera goes back to the player) for the mod `module`, which then no longer holds
/// the free camera. `OK` with nothing sent when no mod holds it; `REFUSED` when another mod holds it. The thread and
/// readiness rules of `set_free` apply. Area `camera.free`.
pub type CameraReleaseFreeFn = unsafe extern "C" fn(module: AmmlStr) -> AmmlStatus;

/// Gives in `out` 1 when the level of the joined world has the `minecraft:free` preset, 0 when it has not. Game thread
/// only (`REFUSED` elsewhere); `NOT_READY` while no world is joined. Area `camera.free`.
pub type CameraFreeAvailableFn = unsafe extern "C" fn(out: *mut u8) -> AmmlStatus;

/// Gives in `out` the vanilla camera that `client` shows the world through: the camera with the game's active camera
/// component, named `minecraft:first_person`, `minecraft:third_person` or `minecraft:third_person_front` (any other
/// name, or a camera with no name the loader can read, is `AmmlCameraKind::OTHER`). Game thread only, for the primary
/// client (`REFUSED` elsewhere and while no world is joined); `NOT_AVAILABLE` when the area is not offered; `FAILED`
/// when the game gives no camera registry, or no camera in it holds the active camera component. Area `camera.active`.
pub type CameraActiveFn = unsafe extern "C" fn(client: *const c_void, out: *mut AmmlCameraKind) -> AmmlStatus;

/// The camera functions of `AmmlServices` (areas `camera.*`). The table only grows: a function added later is
/// appended, `size` and `version` grow, and a reader calls a function only when `size` covers it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlCameraTable {
    pub size: u32,
    pub version: u32,
    pub set_pose: CameraSetPoseFn,
    pub rendered_position: CameraRenderedPositionFn,
    pub projection: CameraProjectionFn,
    pub set_free: Option<CameraSetFreeFn>,
    pub release_free: Option<CameraReleaseFreeFn>,
    pub free_available: Option<CameraFreeAvailableFn>,
    pub active_camera: Option<CameraActiveFn>,
}

impl AmmlCameraTable {
    /// `size` of a v1 camera table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 32;
    /// `size` of a v2 camera table: v1 plus `set_free`, `release_free` and `free_available`.
    pub const SIZE_V2: u32 = 56;
    /// `size` of a v3 camera table: v2 plus `active_camera`.
    pub const SIZE_V3: u32 = 64;

    /// A copy of the table at `table`, read as far as its own `size` covers; a later field is `None`. A null table, or
    /// one shorter than v1, gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlCameraTable) -> Option<AmmlCameraTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlCameraTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlCameraTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the whole struct
        // and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: the copied bytes cover every v1 field, written from a valid table; every later field is an `Option`
        // of a function pointer, for which all zero bytes are `None`.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlCameraTable>() == AmmlCameraTable::SIZE_V3 as usize);
