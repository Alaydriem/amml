use std::ffi::c_void;

use amml_abi::{AmmlActorTable, AmmlRotation, AmmlStatus, AmmlVec3};

use crate::{ActorAdapter, Boundary};

const TABLE_VERSION: u32 = 2;

pub(crate) static ACTOR: AmmlActorTable = AmmlActorTable {
    size: AmmlActorTable::SIZE_V2,
    version: TABLE_VERSION,
    is_alive: ActorTable::is_alive,
    position: Some(ActorTable::position),
    eye_position: Some(ActorTable::eye_position),
    rotation: Some(ActorTable::rotation),
    dimension: Some(ActorTable::dimension),
    dimension_id: Some(ActorTable::dimension_id),
};

/// The actor table behind `AmmlServices::actor`, and its `extern "C"` functions. Each function checks its
/// pointers and delegates to `ActorAdapter` inside the boundary, so a panic never crosses it. With no actor adapter
/// installed (a host without one) every call gives `NOT_AVAILABLE`.
pub struct ActorTable;

impl ActorTable {
    unsafe extern "C" fn position(player: *const c_void, out: *mut AmmlVec3) -> AmmlStatus {
        Self::read(out, |adapter| adapter.position(player))
    }

    unsafe extern "C" fn eye_position(player: *const c_void, out: *mut AmmlVec3) -> AmmlStatus {
        Self::read(out, |adapter| adapter.eye_position(player))
    }

    unsafe extern "C" fn rotation(player: *const c_void, out: *mut AmmlRotation) -> AmmlStatus {
        Self::read(out, |adapter| adapter.rotation(player))
    }

    unsafe extern "C" fn dimension(player: *const c_void, out: *mut *const c_void) -> AmmlStatus {
        Self::read(out, |adapter| adapter.dimension(player).map(std::ptr::with_exposed_provenance))
    }

    unsafe extern "C" fn dimension_id(dimension: *const c_void, out: *mut i32) -> AmmlStatus {
        Self::read(out, |adapter| adapter.dimension_id(dimension))
    }

    // Runs `body` on the adapter inside the boundary and writes its value to `out`.
    fn read<T>(out: *mut T, body: impl FnOnce(&ActorAdapter) -> Result<T, AmmlStatus>) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let adapter = ActorAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(body) {
                Ok(value) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(value) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn is_alive(player: *const c_void, out: *mut u8) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let adapter = ActorAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(|adapter| adapter.is_alive(player)) {
                Ok(alive) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(u8::from(alive)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }
}
