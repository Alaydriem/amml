use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use crate::{Boundary, NetworkAdapter, RequestAdapter, ScreenDetours, SessionHook, UiFlags};

/// `void ClientNetworkHandler::handle(NetworkIdentifier const&, <Packet> const&)`: rcx the handler, rdx the sender
/// identifier, r8 the packet; no result. The same shape for every packet the loader watches.
type HandleFn = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);

static OPEN_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static CLOSE_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static RESPONSE_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static LIST_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SKIN_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on the handlers of the packets the request path watches, one for each slot of the client's
/// network handler vtable: `ContainerOpen` and `ContainerClose` (the server's open of the hidden inventory and its
/// answer to our close are hidden from the game) and `ItemStackResponse` (read first, always given to the game).
///
/// Each request detour loads `UiFlags` once. With `REQUEST` clear (no request runs and no window for a hidden answer is
/// open) it calls the original with the registers it received and returns.
///
/// The `PlayerList` and `PlayerSkin` detours call the original first. Then, with `PLAYER_LIST` or `PLAYER_SKIN` set (a
/// listener of `network.player_list` or `network.player_skin` exists), they hand the packet to `NetworkAdapter`, which
/// raises the event while the packet is live. With the bit clear the whole detour is the original call and one relaxed
/// load.
pub struct PacketDetours;

impl PacketDetours {
    /// The vtable global that holds the handler slots.
    pub const VTABLE: &'static str = "LegacyClientNetworkHandler.vtable";
    pub const OPEN: &'static str = "ClientNetworkHandler.handle.ContainerOpenPacket";
    pub const CLOSE: &'static str = "ClientNetworkHandler.handle.ContainerClosePacket";
    pub const RESPONSE: &'static str = "ClientNetworkHandler.handle.ItemStackResponsePacket";
    pub const LIST: &'static str = "ClientNetworkHandler.handle.PlayerListPacket";
    pub const SKIN: &'static str = "ClientNetworkHandler.handle.PlayerSkinPacket";

    pub fn open_hook() -> SessionHook {
        Self::hook(Self::OPEN, Self::open as HandleFn as *const c_void, &OPEN_ORIGINAL)
    }

    pub fn close_hook() -> SessionHook {
        Self::hook(Self::CLOSE, Self::close as HandleFn as *const c_void, &CLOSE_ORIGINAL)
    }

    pub fn response_hook() -> SessionHook {
        Self::hook(Self::RESPONSE, Self::response as HandleFn as *const c_void, &RESPONSE_ORIGINAL)
    }

    pub fn list_hook() -> SessionHook {
        Self::hook(Self::LIST, Self::list as HandleFn as *const c_void, &LIST_ORIGINAL)
    }

    pub fn skin_hook() -> SessionHook {
        Self::hook(Self::SKIN, Self::skin as HandleFn as *const c_void, &SKIN_ORIGINAL)
    }

    /// Sets whether a listener of `PlayerListUpdate` exists.
    pub(crate) fn set_list_wanted(wanted: bool) {
        UiFlags::put(UiFlags::PLAYER_LIST, wanted);
    }

    /// Sets whether a listener of `PlayerSkinChanged` exists.
    pub(crate) fn set_skin_wanted(wanted: bool) {
        UiFlags::put(UiFlags::PLAYER_SKIN, wanted);
    }

    fn hook(entry: &'static str, detour: *const c_void, original: &'static AtomicPtr<c_void>) -> SessionHook {
        SessionHook { entry, vtable: Some(Self::VTABLE), detour, original }
    }

    unsafe extern "C" fn open(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
        if UiFlags::load() & UiFlags::REQUEST != 0 {
            let hidden = Boundary::run(false, || {
                RequestAdapter::with(false, |adapter| adapter.container_open(packet.expose_provenance()))
            });
            if hidden {
                return;
            }
        }
        // SAFETY: the hook on the ContainerOpen handler is installed, so its slot holds a `HandleFn`.
        let original = unsafe { ScreenDetours::original::<HandleFn>(&OPEN_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, source, packet) };
    }

    unsafe extern "C" fn close(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
        if UiFlags::load() & UiFlags::REQUEST != 0 {
            let hidden = Boundary::run(false, || {
                RequestAdapter::with(false, |adapter| adapter.container_close(packet.expose_provenance()))
            });
            if hidden {
                return;
            }
        }
        // SAFETY: the hook on the ContainerClose handler is installed, so its slot holds a `HandleFn`.
        let original = unsafe { ScreenDetours::original::<HandleFn>(&CLOSE_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, source, packet) };
    }

    unsafe extern "C" fn response(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
        if UiFlags::load() & UiFlags::REQUEST != 0 {
            Boundary::run((), || {
                RequestAdapter::with((), |adapter| adapter.item_stack_response(packet.expose_provenance()));
            });
        }
        // SAFETY: the hook on the ItemStackResponse handler is installed, so its slot holds a `HandleFn`.
        let original = unsafe { ScreenDetours::original::<HandleFn>(&RESPONSE_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, source, packet) };
    }

    unsafe extern "C" fn list(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
        // SAFETY: the hook on the PlayerList handler is installed, so its slot holds a `HandleFn`.
        let original = unsafe { ScreenDetours::original::<HandleFn>(&LIST_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, source, packet) };
        if UiFlags::load() & UiFlags::PLAYER_LIST != 0 {
            Boundary::run((), || NetworkAdapter::with((), |adapter| adapter.player_list(packet.expose_provenance())));
        }
    }

    unsafe extern "C" fn skin(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
        // SAFETY: the hook on the PlayerSkin handler is installed, so its slot holds a `HandleFn`.
        let original = unsafe { ScreenDetours::original::<HandleFn>(&SKIN_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, source, packet) };
        if UiFlags::load() & UiFlags::PLAYER_SKIN != 0 {
            Boundary::run((), || NetworkAdapter::with((), |adapter| adapter.player_skin(packet.expose_provenance())));
        }
    }
}
