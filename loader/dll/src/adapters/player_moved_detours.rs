use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use crate::{
    Boundary, HandlerDetours, MovePlayerLayout, PacketDetours, PlayerMovedAdapter, ScreenDetours, SessionHook,
};

/// `void ClientNetworkHandler::handle(NetworkIdentifier const&, MovePlayerPacket const&)`: rcx the handler, rdx the
/// sender identifier, r8 the packet; no result.
type HandleFn = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);

static ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// A mod listens to `PlayerMoved`. False: the detour calls the original and nothing else.
static WANTED: AtomicBool = AtomicBool::new(false);

/// The loader's detour on the `MovePlayerPacket` handler of the client's network handler (a slot of
/// `LegacyClientNetworkHandler.vtable`, patched the same way as the other handler slots), for `PlayerMoved`.
///
/// The detour first gives its handler to `HandlerDetours::note`. With no listener it reads one more relaxed flag and
/// calls the original. With a listener it reads the packet's fields before the game's handler and raises the event
/// After it. A panic inside the loader's work is caught at the boundary; the game's own call runs once with its own
/// arguments.
pub struct PlayerMovedDetours;

impl PlayerMovedDetours {
    pub const ENTRY: &'static str = "ClientNetworkHandler.handle.MovePlayerPacket";

    /// The hook on the `MovePlayerPacket` handler slot.
    pub fn hook() -> SessionHook {
        SessionHook {
            entry: Self::ENTRY,
            vtable: Some(PacketDetours::VTABLE),
            detour: Self::handle as HandleFn as *const c_void,
            original: &ORIGINAL,
        }
    }

    /// Sets whether a mod listens to `PlayerMoved`.
    pub fn set_wanted(wanted: bool) {
        WANTED.store(wanted, Ordering::Release);
    }

    unsafe extern "C" fn handle(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
        HandlerDetours::note(Self::ENTRY, this, source);
        let fields = if WANTED.load(Ordering::Relaxed) && !packet.is_null() {
            // SAFETY: `packet` is the live `MovePlayerPacket` the game hands its handler; the mirror covers the fields
            // on every supported build.
            Some(unsafe { MovePlayerLayout::read(packet.expose_provenance()) })
        } else {
            None
        };
        // SAFETY: the hook on the MovePlayerPacket handler is installed, so its slot holds a `HandleFn`.
        let original = unsafe { ScreenDetours::original::<HandleFn>(&ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, source, packet) };
        if let Some(fields) = fields {
            Boundary::run((), || PlayerMovedAdapter::with((), |adapter| adapter.moved(fields)));
        }
    }
}
