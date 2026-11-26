use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::time::Instant;

use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;
use windows::Win32::Graphics::Dxgi::{DXGI_ERROR_INVALID_CALL, DXGI_PRESENT_PARAMETERS};
use windows::core::HRESULT;

use super::{OverlayError, OverlayShared, PresentDepth, ProbeTargets, ResizeDepth};
use crate::LoaderState;

type PresentFn = unsafe extern "system" fn(*mut c_void, u32, u32) -> HRESULT;
type Present1Fn = unsafe extern "system" fn(*mut c_void, u32, u32, *const DXGI_PRESENT_PARAMETERS) -> HRESULT;
type ResizeBuffersFn = unsafe extern "system" fn(*mut c_void, u32, u32, u32, DXGI_FORMAT, u32) -> HRESULT;
type ResizeBuffers1Fn =
    unsafe extern "system" fn(*mut c_void, u32, u32, u32, DXGI_FORMAT, u32, *const u32, *const *mut c_void) -> HRESULT;
type ExecuteCommandListsFn = unsafe extern "system" fn(*mut c_void, u32, *const *mut c_void);

static ORIGINAL_PRESENT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ORIGINAL_PRESENT1: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ORIGINAL_RESIZE_BUFFERS: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ORIGINAL_RESIZE_BUFFERS1: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ORIGINAL_EXECUTE_COMMAND_LISTS: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The five DXGI and D3D12 detours and their install as loader hooks. Each detour runs its overlay part through
/// `OverlayShared` and then calls the original exactly once, also after a panic, before the hooks are armed and
/// while the loader shuts down.
pub struct PresentHooks;

impl PresentHooks {
    // Loader hooks of the overlay take the default place in each chain.
    const PRIORITY: i32 = 0;

    /// Hooks the entries of `targets`. The queue entry goes first, so the D3D12 queue can be seen before any Present,
    /// then both resizes, then both presents, so no chain is bound before its resize is hooked. The queue entry is
    /// optional; a required entry that fails ends the install, and the hooks already in place stay and only call the
    /// original, because the overlay is never armed. Gives whether the queue entry is hooked.
    pub fn install(state: &'static LoaderState, targets: &ProbeTargets) -> Result<bool, OverlayError> {
        let mut queue_hooked = false;
        if let Some(target) = targets.execute_command_lists {
            let detour = Self::execute_command_lists as ExecuteCommandListsFn as *const c_void;
            match Self::hook(state, "ExecuteCommandLists", target, detour, &ORIGINAL_EXECUTE_COMMAND_LISTS) {
                Ok(()) => queue_hooked = true,
                Err(error) => {
                    curia::warn!("overlay has no queue hook, the queue of a D3D12 game is not seen", {
                        error: error.to_string()
                    });
                }
            }
        }
        let required: [(&'static str, usize, *const c_void, &'static AtomicPtr<c_void>); 4] = [
            (
                "ResizeBuffers",
                targets.resize_buffers,
                Self::resize_buffers as ResizeBuffersFn as *const c_void,
                &ORIGINAL_RESIZE_BUFFERS,
            ),
            (
                "ResizeBuffers1",
                targets.resize_buffers1,
                Self::resize_buffers1 as ResizeBuffers1Fn as *const c_void,
                &ORIGINAL_RESIZE_BUFFERS1,
            ),
            ("Present", targets.present, Self::present as PresentFn as *const c_void, &ORIGINAL_PRESENT),
            ("Present1", targets.present1, Self::present1 as Present1Fn as *const c_void, &ORIGINAL_PRESENT1),
        ];
        for (entry, target, detour, original) in required {
            Self::hook(state, entry, target, detour, original)?;
        }
        Ok(queue_hooked)
    }

    fn hook(
        state: &'static LoaderState,
        entry: &'static str,
        target: usize,
        detour: *const c_void,
        original: &'static AtomicPtr<c_void>,
    ) -> Result<(), OverlayError> {
        let target = std::ptr::with_exposed_provenance::<c_void>(target);
        // SAFETY: `target` is the entry `entry` read from the vtable of a live probe object (or null, which the engine
        // refuses), `detour` has that entry's signature, and `original` is a static `AtomicPtr` that is read only
        // through the atomic.
        unsafe { state.install_loader_hook(target, detour, Self::PRIORITY, original.as_ptr().cast::<*const c_void>()) }
            .map(|_| ())
            .map_err(|status| OverlayError::Hook { entry, status: status.0 })
    }

    unsafe extern "system" fn present(chain: *mut c_void, sync: u32, flags: u32) -> HRESULT {
        let depth = PresentDepth::enter();
        let shared = OverlayShared::global();
        let frame = shared.before_present(chain, sync, flags, depth.outermost());
        let Some(original) = Self::original::<PresentFn>(&ORIGINAL_PRESENT) else {
            return DXGI_ERROR_INVALID_CALL;
        };
        let started = Instant::now();
        // SAFETY: the original takes the arguments DXGI gave this detour.
        let result = unsafe { original(chain, sync, flags) };
        if let Some(frame) = frame {
            shared.after_present(frame, result, started.elapsed());
        }
        result
    }

    unsafe extern "system" fn present1(
        chain: *mut c_void,
        sync: u32,
        flags: u32,
        params: *const DXGI_PRESENT_PARAMETERS,
    ) -> HRESULT {
        let depth = PresentDepth::enter();
        let shared = OverlayShared::global();
        let frame = shared.before_present(chain, sync, flags, depth.outermost());
        let Some(original) = Self::original::<Present1Fn>(&ORIGINAL_PRESENT1) else {
            return DXGI_ERROR_INVALID_CALL;
        };
        let started = Instant::now();
        // SAFETY: as for `present`.
        let result = unsafe { original(chain, sync, flags, params) };
        if let Some(frame) = frame {
            shared.after_present(frame, result, started.elapsed());
        }
        result
    }

    unsafe extern "system" fn resize_buffers(
        chain: *mut c_void,
        count: u32,
        width: u32,
        height: u32,
        format: DXGI_FORMAT,
        flags: u32,
    ) -> HRESULT {
        let depth = ResizeDepth::enter();
        let shared = OverlayShared::global();
        let given = if depth.outermost() { shared.before_resize(chain, count, width, height) } else { None };
        let (pass_width, pass_height) = given.unwrap_or((width, height));
        let Some(original) = Self::original::<ResizeBuffersFn>(&ORIGINAL_RESIZE_BUFFERS) else {
            return DXGI_ERROR_INVALID_CALL;
        };
        // SAFETY: as for `present`, with the size the overlay gave.
        let result = unsafe { original(chain, count, pass_width, pass_height, format, flags) };
        if given.is_some() {
            shared.after_resize(chain, result);
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "system" fn resize_buffers1(
        chain: *mut c_void,
        count: u32,
        width: u32,
        height: u32,
        format: DXGI_FORMAT,
        flags: u32,
        node_masks: *const u32,
        queues: *const *mut c_void,
    ) -> HRESULT {
        let depth = ResizeDepth::enter();
        let shared = OverlayShared::global();
        let given = if depth.outermost() { shared.before_resize(chain, count, width, height) } else { None };
        let (pass_width, pass_height) = given.unwrap_or((width, height));
        let Some(original) = Self::original::<ResizeBuffers1Fn>(&ORIGINAL_RESIZE_BUFFERS1) else {
            return DXGI_ERROR_INVALID_CALL;
        };
        // SAFETY: as for `resize_buffers`.
        let result = unsafe { original(chain, count, pass_width, pass_height, format, flags, node_masks, queues) };
        if given.is_some() {
            shared.after_resize(chain, result);
        }
        result
    }

    unsafe extern "system" fn execute_command_lists(queue: *mut c_void, count: u32, lists: *const *mut c_void) {
        OverlayShared::global().offer_queue(queue);
        if let Some(original) = Self::original::<ExecuteCommandListsFn>(&ORIGINAL_EXECUTE_COMMAND_LISTS) {
            // SAFETY: as for `present`.
            unsafe { original(queue, count, lists) };
        }
    }

    // `F` is the function pointer type of the entry the slot belongs to.
    fn original<F: Copy>(slot: &AtomicPtr<c_void>) -> Option<F> {
        // Acquire pairs with the engine's store of the slot, made before the detour was enabled.
        let original = slot.load(Ordering::Acquire);
        if original.is_null() {
            return None;
        }
        // SAFETY: every slot holds a function of the type its detour casts it to, and `F` is a function pointer of
        // the size of a pointer.
        Some(unsafe { std::mem::transmute_copy::<*mut c_void, F>(&original) })
    }
}
