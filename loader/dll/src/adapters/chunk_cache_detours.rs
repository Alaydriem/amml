use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use crate::{Boundary, NetworkAdapter, ScreenDetours, SessionHook, WireDetours};

/// `void NetworkSystem::send(NetworkIdentifier const&, Packet const&, SubClientId)`: rcx the system, rdx the identifier,
/// r8 the packet, r9d the recipient sub-client id; no result (survey section 8, row C6).
type SendFn = unsafe extern "C" fn(*mut c_void, *const c_void, *const c_void, u32);

static SEND_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detour on `NetworkSystem::send`, before the packet is serialized: while a mod wishes the chunk cache
/// off, it writes 0 into `mEnabled` of every `ClientCacheStatusPacket` the game sends.
///
/// The detour shares the flags word of `WireDetours` (`WireDetours::CHUNK_CACHE`): with the flag clear it does one
/// relaxed load and calls the original, with no lock, no allocation and no log. A panic inside the loader's work is
/// caught at the boundary; the game's own call runs once with its own arguments.
pub struct ChunkCacheDetours;

impl ChunkCacheDetours {
    pub const SEND: &'static str = "NetworkSystem.send";

    /// The hook on `NetworkSystem::send`.
    pub fn send_hook() -> SessionHook {
        SessionHook {
            entry: Self::SEND,
            vtable: None,
            detour: Self::send as SendFn as *const c_void,
            original: &SEND_ORIGINAL,
        }
    }

    unsafe extern "C" fn send(this: *mut c_void, id: *const c_void, packet: *const c_void, sub: u32) {
        if WireDetours::flags() & WireDetours::CHUNK_CACHE != 0 {
            Boundary::run((), || {
                NetworkAdapter::with((), |adapter| adapter.edit_cache_status(packet.expose_provenance()));
            });
        }
        // SAFETY: the hook on `send` is installed, so its original slot holds a `SendFn`.
        let original = unsafe { ScreenDetours::original::<SendFn>(&SEND_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, id, packet, sub) };
    }
}
