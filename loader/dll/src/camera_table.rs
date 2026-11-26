use std::ffi::c_void;

use amml_abi::{
    AmmlActorTarget, AmmlCameraKind, AmmlCameraTable, AmmlPose, AmmlProjection, AmmlStatus, AmmlStr, AmmlVec3,
};

use crate::{ActiveCameraAdapter, BodyHideAdapter, Boundary, CameraAdapter, FreeCameraAdapter, LoaderState};

const TABLE_VERSION: u32 = 4;

pub(crate) static CAMERA: AmmlCameraTable = AmmlCameraTable {
    size: AmmlCameraTable::SIZE_V4,
    version: TABLE_VERSION,
    set_pose: CameraTable::set_pose,
    rendered_position: CameraTable::rendered_position,
    projection: CameraTable::projection,
    set_free: Some(CameraTable::set_free),
    release_free: Some(CameraTable::release_free),
    free_available: Some(CameraTable::free_available),
    active_camera: Some(CameraTable::active_camera),
    hide_actor: Some(CameraTable::hide_actor),
};

/// The camera table behind `AmmlServices::camera`, and its `extern "C"` functions. Each function checks its pointers
/// and delegates to `CameraAdapter` or `FreeCameraAdapter` inside the boundary, so a panic never crosses it. With no
/// adapter installed (a host without one) every call of its functions gives `NOT_AVAILABLE`. The free camera calls and
/// `active_camera` follow the game thread rule of a `CameraSetup` listener first (`CameraAdapter::check_game_call`).
pub struct CameraTable;

impl CameraTable {
    unsafe extern "C" fn set_pose(module: AmmlStr, frame: u64, pose: *const AmmlPose) -> AmmlStatus {
        Boundary::guard(|| {
            if pose.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `pose` is non-null and the caller promises it is readable for the call.
            let pose = unsafe { pose.read_unaligned() };
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match CameraAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE).and_then(|a| a.set_pose(owner, frame, pose)) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn rendered_position(frame: u64, out: *mut AmmlVec3) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match CameraAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE).and_then(|a| a.rendered_position(frame)) {
                Ok([x, y, z]) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(AmmlVec3 { x, y, z }) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn projection(frame: u64, out: *mut AmmlProjection) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match CameraAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE).and_then(|a| a.projection(frame)) {
                Ok((fov_y, aspect)) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(AmmlProjection { fov_y, aspect }) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn set_free(module: AmmlStr, pose: *const AmmlPose) -> AmmlStatus {
        Boundary::guard(|| {
            if pose.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `pose` is non-null and the caller promises it is readable for the call.
            let pose = unsafe { pose.read_unaligned() };
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let set = CameraAdapter::check_game_call()
                .and_then(|()| FreeCameraAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.set_free(owner, pose));
            match set {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn release_free(module: AmmlStr) -> AmmlStatus {
        Boundary::guard(|| {
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let released = CameraAdapter::check_game_call()
                .and_then(|()| FreeCameraAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.release_free(owner));
            match released {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn free_available(out: *mut u8) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let available = CameraAdapter::check_game_call()
                .and_then(|()| FreeCameraAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(FreeCameraAdapter::free_available);
            match available {
                Ok(available) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(u8::from(available)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn active_camera(client: *const c_void, out: *mut AmmlCameraKind) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let kind = CameraAdapter::check_game_call()
                .and_then(|()| ActiveCameraAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.active_camera(client));
            match kind {
                Ok(kind) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(kind) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn hide_actor(module: AmmlStr, target: AmmlActorTarget, on: u8) -> AmmlStatus {
        Boundary::guard(|| {
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match BodyHideAdapter::get()
                .ok_or(AmmlStatus::NOT_AVAILABLE)
                .and_then(|a| a.hide_actor(owner, target, on != 0))
            {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    // The index of the live mod `module`.
    fn owner(module: AmmlStr) -> Result<usize, AmmlStatus> {
        let state = LoaderState::get().ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the caller promises `module` is valid for the call.
        let module = unsafe { module.as_str() }?;
        state.live_mod(module)
    }
}
