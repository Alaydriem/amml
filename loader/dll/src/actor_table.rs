use std::ffi::c_void;

use amml_abi::{
    AmmlAabb, AmmlActorTable, AmmlBlockPos, AmmlFace, AmmlGameType, AmmlHitResult, AmmlPermissionLevel, AmmlRotation,
    AmmlStatus, AmmlStr, AmmlVec3,
};

use crate::{ActorAdapter, Boundary, GameModeAdapter, LoaderState};

const TABLE_VERSION: u32 = 6;

pub(crate) static ACTOR: AmmlActorTable = AmmlActorTable {
    size: AmmlActorTable::SIZE_V6,
    version: TABLE_VERSION,
    is_alive: ActorTable::is_alive,
    position: Some(ActorTable::position),
    eye_position: Some(ActorTable::eye_position),
    rotation: Some(ActorTable::rotation),
    dimension: Some(ActorTable::dimension),
    dimension_id: Some(ActorTable::dimension_id),
    head_position: Some(ActorTable::head_position),
    block_source: Some(ActorTable::block_source),
    health: Some(ActorTable::health),
    view_vector: Some(ActorTable::view_vector),
    aabb: Some(ActorTable::aabb),
    game_type: Some(ActorTable::game_type),
    permission_level: Some(ActorTable::permission_level),
    set_rotation: Some(ActorTable::set_rotation),
    build_at: Some(ActorTable::build_at),
    build_delay: Some(ActorTable::build_delay),
    max_pick_range: Some(ActorTable::max_pick_range),
    hit_result: Some(ActorTable::hit_result),
};

/// The actor table behind `AmmlServices::actor`, and its `extern "C"` functions. Each function checks its
/// pointers and delegates to `ActorAdapter` inside the boundary, so a panic never crosses it. With no actor adapter
/// installed (a host without one) every call gives `NOT_AVAILABLE`. The build and pick range functions delegate to
/// `GameModeAdapter` the same way.
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

    unsafe extern "C" fn head_position(player: *const c_void, out: *mut AmmlVec3) -> AmmlStatus {
        Self::read(out, |adapter| adapter.head_position(player))
    }

    unsafe extern "C" fn block_source(player: *const c_void, out: *mut *const c_void) -> AmmlStatus {
        Self::read(out, |adapter| adapter.block_source(player).map(std::ptr::with_exposed_provenance))
    }

    unsafe extern "C" fn health(player: *const c_void, out: *mut i32) -> AmmlStatus {
        Self::read(out, |adapter| adapter.health(player))
    }

    unsafe extern "C" fn view_vector(player: *const c_void, out: *mut AmmlVec3) -> AmmlStatus {
        Self::read(out, |adapter| adapter.view_vector(player))
    }

    unsafe extern "C" fn aabb(player: *const c_void, out: *mut AmmlAabb) -> AmmlStatus {
        Self::read(out, |adapter| adapter.aabb(player))
    }

    unsafe extern "C" fn game_type(player: *const c_void, out: *mut AmmlGameType) -> AmmlStatus {
        Self::read(out, |adapter| adapter.game_type(player))
    }

    unsafe extern "C" fn permission_level(player: *const c_void, out: *mut AmmlPermissionLevel) -> AmmlStatus {
        Self::read(out, |adapter| adapter.permission_level(player))
    }

    unsafe extern "C" fn set_rotation(
        player: *const c_void,
        rotation: *const AmmlRotation,
        head_yaw: f32,
        has_head_yaw: u8,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if rotation.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `rotation` is non-null and the caller promises it is readable.
            let rotation = unsafe { rotation.read() };
            let head = (has_head_yaw != 0).then_some(head_yaw);
            let adapter = ActorAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(|adapter| adapter.set_rotation(player, rotation, head)) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn build_at(
        module: AmmlStr,
        player: *const c_void,
        pos: *const AmmlBlockPos,
        face: AmmlFace,
        click: *const AmmlVec3,
        out: *mut u8,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if pos.is_null() || click.is_null() || out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `pos` and `click` are non-null and the caller promises they are readable.
            let (pos, click) = unsafe { (pos.read_unaligned(), click.read_unaligned()) };
            let built = Self::owner(module).and_then(|owner| {
                let adapter = GameModeAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
                adapter.build_at(owner, player, pos, face, click)
            });
            match built {
                Ok(built) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(u8::from(built)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn build_delay(player: *const c_void, out: *mut u32) -> AmmlStatus {
        Self::game_mode_read(out, |adapter| adapter.build_delay(player))
    }

    unsafe extern "C" fn max_pick_range(player: *const c_void, out: *mut f32) -> AmmlStatus {
        Self::game_mode_read(out, |adapter| adapter.max_pick_range(player))
    }

    unsafe extern "C" fn hit_result(player: *const c_void, out: *mut AmmlHitResult) -> AmmlStatus {
        Self::game_mode_read(out, |adapter| adapter.hit_result(player))
    }

    // The index of the live mod `module`.
    fn owner(module: AmmlStr) -> Result<usize, AmmlStatus> {
        let state = LoaderState::get().ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the caller promises `module` is valid for the call.
        let module = unsafe { module.as_str() }?;
        state.live_mod(module)
    }

    // Runs `body` on the game mode adapter inside the boundary and writes its value to `out`.
    fn game_mode_read<T>(out: *mut T, body: impl FnOnce(&GameModeAdapter) -> Result<T, AmmlStatus>) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let adapter = GameModeAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
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
