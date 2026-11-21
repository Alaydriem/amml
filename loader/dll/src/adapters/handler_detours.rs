use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

use crate::{Boundary, FreeCameraAdapter, PacketDetours, ScreenDetours, SessionHook};

/// `void ClientNetworkHandler::handle(NetworkIdentifier const&, <Packet> const&)`: rcx the handler, rdx the sender
/// identifier, r8 the packet; no result.
type HandleFn = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);

static CAMERA_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static CHUNK_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// What the handler detours have work for besides the game's own call. Zero: each detour calls the original and nothing
// else.
static WANTED: AtomicU32 = AtomicU32::new(0);

/// The loader's detours on the `CameraInstructionPacket` and `NetworkChunkPublisherUpdatePacket` handlers of the
/// client's network handler (slots of `LegacyClientNetworkHandler.vtable`), and the capture note of every hooked
/// handler slot.
///
/// Each detour reads one flags word. With no flag set it calls the original and does nothing else: no lock, no
/// allocation, no log. `CAPTURE` is set while an enabled mod declares `camera.free` or `world.shapes`: every hooked
/// handler slot then gives its handler and identifier to `FreeCameraAdapter`. `LISTENER` is set while a mod listens to
/// `CameraInstructionReceived`: the camera detour then raises it Before the game's handler. `HELD` is set while a mod
/// holds the free camera: the camera detour then ends the hold on a server instruction that sets or clears the camera.
/// A panic inside the loader's work is caught at the boundary; the game's own call runs once with its own arguments.
pub struct HandlerDetours;

impl HandlerDetours {
    pub const CAMERA: &'static str = "ClientNetworkHandler.handle.CameraInstructionPacket";
    pub const CHUNK: &'static str = "ClientNetworkHandler.handle.NetworkChunkPublisherUpdatePacket";
    /// An enabled mod declares `camera.free` or `world.shapes`: the handler slots capture their handler.
    pub const CAPTURE: u32 = 1;
    /// A mod listens to `CameraInstructionReceived`.
    pub const LISTENER: u32 = 2;
    /// A mod holds the free camera: the camera detour reads whether a server instruction sets or clears the camera.
    pub const HELD: u32 = 4;

    /// The two handler hooks, in install order.
    pub fn hooks() -> [SessionHook; 2] {
        [
            SessionHook {
                entry: Self::CAMERA,
                vtable: Some(PacketDetours::VTABLE),
                detour: Self::camera as HandleFn as *const c_void,
                original: &CAMERA_ORIGINAL,
            },
            SessionHook {
                entry: Self::CHUNK,
                vtable: Some(PacketDetours::VTABLE),
                detour: Self::chunk as HandleFn as *const c_void,
                original: &CHUNK_ORIGINAL,
            },
        ]
    }

    /// Sets or clears `flag`.
    pub fn set(flag: u32, on: bool) {
        if on {
            WANTED.fetch_or(flag, Ordering::AcqRel);
        } else {
            WANTED.fetch_and(!flag, Ordering::AcqRel);
        }
    }

    /// A hooked handler slot runs on `handler` with the identifier `source`, from `entry`: one relaxed load, and the
    /// capture while `CAPTURE` is set.
    pub(crate) fn note(entry: &'static str, handler: *mut c_void, source: *mut c_void) {
        if WANTED.load(Ordering::Relaxed) & Self::CAPTURE == 0 {
            return;
        }
        let (handler, source) = (handler.expose_provenance(), source.expose_provenance());
        Boundary::run((), || FreeCameraAdapter::with((), |adapter| adapter.capture(entry, handler, source)));
    }

    unsafe extern "C" fn camera(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
        let wanted = WANTED.load(Ordering::Relaxed);
        if wanted != 0 {
            let (handler, id, packet) =
                (this.expose_provenance(), source.expose_provenance(), packet.expose_provenance());
            Boundary::run((), || {
                FreeCameraAdapter::with((), |adapter| adapter.instruction_seen(wanted, handler, id, packet));
            });
        }
        // SAFETY: the hook on the CameraInstructionPacket handler is installed, so its slot holds a `HandleFn`.
        let original = unsafe { ScreenDetours::original::<HandleFn>(&CAMERA_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, source, packet) };
    }

    unsafe extern "C" fn chunk(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
        Self::note(Self::CHUNK, this, source);
        // SAFETY: the hook on the NetworkChunkPublisherUpdatePacket handler is installed, so its slot holds a
        // `HandleFn`.
        let original = unsafe { ScreenDetours::original::<HandleFn>(&CHUNK_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, source, packet) };
    }
}
