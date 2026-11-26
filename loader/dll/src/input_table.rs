use amml_abi::{
    AmmlInputBlocks, AmmlInputTable, AmmlLeaseMode, AmmlLeaseRequest, AmmlLeaseState, AmmlModifiers, AmmlMouseButton,
    AmmlMoveIntent, AmmlOtherLeases, AmmlStatus, AmmlStr,
};
use amml_runtime::{LeaseTerms, ResourceId};

use crate::{Boundary, LoaderState};

const TABLE_VERSION: u32 = 3;

pub(crate) static INPUT: AmmlInputTable = AmmlInputTable {
    size: AmmlInputTable::SIZE_V3,
    version: TABLE_VERSION,
    modifiers: InputTable::modifiers,
    is_down: InputTable::is_down,
    acquire: InputTable::acquire,
    lease_state: InputTable::lease_state,
    key_down: InputTable::key_down,
    button_down: InputTable::button_down,
    release_hold: InputTable::release_hold,
    wheel: InputTable::wheel,
    turn: InputTable::turn,
    set_move: InputTable::set_move,
    release_all: InputTable::release_all,
    end_lease: InputTable::end_lease,
    set_ranking: InputTable::set_ranking,
    set_coexist: Some(InputTable::set_coexist),
    held_by_others: Some(InputTable::held_by_others),
};

/// The input table behind `AmmlServices::input`, and its `extern "C"` functions. Each function reads
/// its arguments and delegates to `InputService` inside the boundary, so a panic never crosses it.
pub struct InputTable;

impl InputTable {
    // Most passthrough keys and ranked ids one call may carry; more is an invalid argument.
    const MAX_PASSTHROUGH: usize = 0x100;
    const MAX_RANKED: usize = 0x400;

    unsafe extern "C" fn modifiers() -> AmmlModifiers {
        Boundary::run(AmmlModifiers(0), || {
            LoaderState::get().map_or(AmmlModifiers(0), |state| state.input().keys().modifiers())
        })
    }

    unsafe extern "C" fn is_down(key: u16, out: *mut u8) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out.is_null() || key >= 0x100 {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let Some(down) = state.input().keys().is_down(key) else {
                return AmmlStatus::NOT_READY;
            };
            // SAFETY: `out` is non-null and the caller promises it is writable.
            unsafe { out.write(u8::from(down)) };
            AmmlStatus::OK
        })
    }

    unsafe extern "C" fn acquire(module: AmmlStr, request: *const AmmlLeaseRequest, out_lease: *mut u64) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if request.is_null() || out_lease.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `module` is valid for the call.
            let module = match unsafe { module.as_str() } {
                Ok(module) => module,
                Err(status) => return status,
            };
            // SAFETY: `request` is non-null and the caller promises it points to a lease request of
            // at least its own `size`, with `passthrough` valid for `passthrough_count` keys.
            let terms = match unsafe { Self::terms(request) } {
                Ok(terms) => terms,
                Err(status) => return status,
            };
            let owner = match state.live_mod(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match state.input().acquire(owner, module, terms) {
                Ok(lease) => {
                    // SAFETY: `out_lease` is non-null and the caller promises it is writable.
                    unsafe { out_lease.write(lease.0) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    /// The terms of a lease request. A `size` below v1, an unknown mode, a null passthrough with a
    /// count or a count above `MAX_PASSTHROUGH` give `INVALID_ARGUMENT`; block bits the loader does
    /// not know are left out.
    ///
    /// # Safety
    /// `request` is non-null and readable for its `size`, at least 4 bytes; with a count, a non-null
    /// `passthrough` holds `passthrough_count` keys valid for the call.
    unsafe fn terms(request: *const AmmlLeaseRequest) -> Result<LeaseTerms, AmmlStatus> {
        // SAFETY: the caller guarantees the first four bytes, which hold `size`, are readable.
        let size = unsafe { request.cast::<u32>().read_unaligned() };
        if size < AmmlLeaseRequest::SIZE_V1 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        // SAFETY: `size` covers the whole v1 request.
        let request = unsafe { request.read_unaligned() };
        if request.mode != AmmlLeaseMode::SHARED && request.mode != AmmlLeaseMode::EXCLUSIVE {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let count = request.passthrough_count;
        let passthrough = if count == 0 {
            Vec::new()
        } else {
            if request.passthrough.is_null() || count > Self::MAX_PASSTHROUGH {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            // SAFETY: the caller guarantees `passthrough` holds `count` keys; it is non-null here.
            unsafe { std::slice::from_raw_parts(request.passthrough, count) }.to_vec()
        };
        Ok(LeaseTerms {
            mode: request.mode,
            blocks: AmmlInputBlocks(request.blocks.0 & AmmlInputBlocks::FULL.0),
            passthrough,
            preempt: request.preempt != 0,
        })
    }

    unsafe extern "C" fn lease_state(lease: u64) -> AmmlLeaseState {
        Boundary::run(AmmlLeaseState::LOST, || {
            LoaderState::get().map_or(AmmlLeaseState::LOST, |state| state.input().lease_state(ResourceId(lease)))
        })
    }

    unsafe extern "C" fn key_down(lease: u64, key: u16, out_hold: *mut u64) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out_hold.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match state.input().key_down(ResourceId(lease), key) {
                Ok(hold) => {
                    // SAFETY: `out_hold` is non-null and the caller promises it is writable.
                    unsafe { out_hold.write(hold.0) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn button_down(lease: u64, button: AmmlMouseButton, out_hold: *mut u64) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out_hold.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match state.input().button_down(ResourceId(lease), button) {
                Ok(hold) => {
                    // SAFETY: `out_hold` is non-null and the caller promises it is writable.
                    unsafe { out_hold.write(hold.0) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn release_hold(hold: u64) -> AmmlStatus {
        Boundary::guard(|| match LoaderState::get() {
            Some(state) => Self::status(state.input().release_hold(ResourceId(hold))),
            None => AmmlStatus::FAILED,
        })
    }

    unsafe extern "C" fn wheel(lease: u64, notches: f32) -> AmmlStatus {
        Boundary::guard(|| match LoaderState::get() {
            Some(state) => Self::status(state.input().wheel(ResourceId(lease), notches)),
            None => AmmlStatus::FAILED,
        })
    }

    unsafe extern "C" fn turn(lease: u64, yaw: f32, pitch: f32) -> AmmlStatus {
        Boundary::guard(|| match LoaderState::get() {
            Some(state) => Self::status(state.input().turn(ResourceId(lease), yaw, pitch)),
            None => AmmlStatus::FAILED,
        })
    }

    unsafe extern "C" fn set_move(lease: u64, intent: *const AmmlMoveIntent) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            // SAFETY: a non-null `intent` points to a move intent the caller keeps valid for the call.
            let intent = (!intent.is_null()).then(|| unsafe { intent.read_unaligned() });
            Self::status(state.input().set_move(ResourceId(lease), intent))
        })
    }

    unsafe extern "C" fn release_all(lease: u64) {
        Boundary::run((), || {
            if let Some(state) = LoaderState::get() {
                state.input().release_all(ResourceId(lease));
            }
        });
    }

    unsafe extern "C" fn end_lease(lease: u64) -> AmmlStatus {
        Boundary::guard(|| match LoaderState::get() {
            Some(state) => Self::status(state.input().end(ResourceId(lease))),
            None => AmmlStatus::FAILED,
        })
    }

    unsafe extern "C" fn set_ranking(module: AmmlStr, ids: *const AmmlStr, count: usize) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            // SAFETY: the caller promises `module` is valid and `ids` holds `count` valid views.
            let (owner, ranked) = match unsafe { Self::policy_args(state, module, ids, count) } {
                Ok(args) => args,
                Err(status) => return status,
            };
            Self::status(state.input().set_ranking(owner, ranked))
        })
    }

    unsafe extern "C" fn set_coexist(module: AmmlStr, ids: *const AmmlStr, count: usize) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if !count.is_multiple_of(2) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `module` is valid and `ids` holds `count` valid views.
            let (owner, ids) = match unsafe { Self::policy_args(state, module, ids, count) } {
                Ok(args) => args,
                Err(status) => return status,
            };
            let pairs = ids.as_chunks::<2>().0.iter().map(|[first, second]| (first.clone(), second.clone())).collect();
            Self::status(state.input().set_coexist(owner, pairs))
        })
    }

    unsafe extern "C" fn held_by_others(module: AmmlStr, out: *mut AmmlOtherLeases) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `module` is valid for the call.
            let owner = match unsafe { module.as_str() }.and_then(|module| state.live_mod(module)) {
                Ok(owner) => owner,
                Err(_) => return AmmlStatus::INVALID_ARGUMENT,
            };
            let (leases, blocks) = state.input().book().active_of_others(owner);
            // SAFETY: `out` is non-null and the caller promises it is writable.
            unsafe { out.write(AmmlOtherLeases { leases, blocks }) };
            AmmlStatus::OK
        })
    }

    /// The live mod `module` and the `count` mod ids in `ids`, for a call that sets the lease policy.
    ///
    /// # Safety
    /// `module` is valid for the call; with a count, a non-null `ids` holds `count` views valid for the call.
    unsafe fn policy_args(
        state: &LoaderState,
        module: AmmlStr,
        ids: *const AmmlStr,
        count: usize,
    ) -> Result<(usize, Vec<String>), AmmlStatus> {
        if count > Self::MAX_RANKED || (count > 0 && ids.is_null()) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        // SAFETY: the caller promises `module` is valid for the call.
        let module = unsafe { module.as_str() }?;
        let views: &[AmmlStr] = if count == 0 {
            &[]
        } else {
            // SAFETY: `ids` is non-null and the caller promises it holds `count` views.
            unsafe { std::slice::from_raw_parts(ids, count) }
        };
        let mut read = Vec::with_capacity(count);
        for view in views {
            // SAFETY: the caller promises each view is valid for the call.
            read.push(unsafe { view.as_str() }?.to_string());
        }
        let owner = state.live_mod(module)?;
        Ok((owner, read))
    }

    fn status(result: Result<(), AmmlStatus>) -> AmmlStatus {
        match result {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        }
    }
}
