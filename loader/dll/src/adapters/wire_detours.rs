use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

use amml_msvc::MsvcString;

use crate::{Boundary, ScreenDetours, SessionHook, WireAdapter};

/// `void NetworkSystem::_sendInternal(NetworkIdentifier const&, Packet const&, std::string const&)`: rcx the system, rdx
/// the identifier, r8 the packet, r9 the serialized bytes; no result (survey section 8, row C1).
type SendFn = unsafe extern "C" fn(*mut c_void, *const c_void, *const c_void, *const MsvcString);
/// `NetworkPeer::DataStatus PacketTraceNetworkPeer::_receivePacket(std::string&, std::shared_ptr<time_point> const&)`:
/// rcx the peer, rdx the out string, r8 the time point, the status in eax (row C3).
type ReceiveFn = unsafe extern "C" fn(*mut c_void, *mut MsvcString, *const c_void) -> i32;

static SEND_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static RECEIVE_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// What the wire detours have work for besides the game's own call. Zero: each detour calls the original and nothing
// else.
static WANTED: AtomicU32 = AtomicU32::new(0);

/// The loader's detours on `NetworkSystem::_sendInternal` (every packet the client and the local server send, before it
/// goes to the connection's batch) and on `PacketTraceNetworkPeer::_receivePacket` (the outermost peer of a connection:
/// one packet per call, after decryption, decompression and the batch split), and the flags word these and
/// `ChunkCacheDetours` read.
///
/// With no flag set, each detour calls the original and does nothing else: one relaxed load, no lock, no allocation, no
/// log. `LISTENER` is set while a mod listens to `PacketWire` or `ConnectionStarted`; only `ChunkCacheDetours` reads
/// `CHUNK_CACHE`. The send detour hands its arguments to `WireAdapter` before the original; the receive detour calls the original first and hands the filled
/// string on only for a status of `HAS_DATA`. A panic inside the loader's work is caught at the boundary; the game's own
/// call runs once with its own arguments.
pub struct WireDetours;

impl WireDetours {
    pub const SEND: &'static str = "NetworkSystem.sendInternal";
    pub const RECEIVE: &'static str = "PacketTraceNetworkPeer.receivePacket";
    /// A mod listens to `PacketWire` or `ConnectionStarted`.
    pub const LISTENER: u32 = 1;
    /// A mod wishes the chunk cache off: `ChunkCacheDetours` writes 0 into every `ClientCacheStatusPacket` it sees.
    pub const CHUNK_CACHE: u32 = 2;
    /// `NetworkPeer::DataStatus::HasData`: the out string holds one packet.
    pub const HAS_DATA: i32 = 0;

    /// The hook on `_sendInternal`.
    pub fn send_hook() -> SessionHook {
        SessionHook {
            entry: Self::SEND,
            vtable: None,
            detour: Self::send as SendFn as *const c_void,
            original: &SEND_ORIGINAL,
        }
    }

    /// The hook on the outer peer's receive.
    pub fn receive_hook() -> SessionHook {
        SessionHook {
            entry: Self::RECEIVE,
            vtable: None,
            detour: Self::receive as ReceiveFn as *const c_void,
            original: &RECEIVE_ORIGINAL,
        }
    }

    /// Sets or clears `flag`.
    pub fn set(flag: u32, on: bool) {
        if on {
            WANTED.fetch_or(flag, Ordering::AcqRel);
        } else {
            WANTED.fetch_and(!flag, Ordering::AcqRel);
        }
    }

    /// The flags word: one relaxed load.
    pub fn flags() -> u32 {
        WANTED.load(Ordering::Relaxed)
    }

    unsafe extern "C" fn send(this: *mut c_void, id: *const c_void, packet: *const c_void, data: *const MsvcString) {
        if Self::flags() & Self::LISTENER != 0 {
            Boundary::run((), || {
                WireAdapter::with((), |adapter| {
                    adapter.outbound(this.expose_provenance(), id.expose_provenance(), data.expose_provenance());
                });
            });
        }
        // SAFETY: the hook on `_sendInternal` is installed, so its original slot holds a `SendFn`.
        let original = unsafe { ScreenDetours::original::<SendFn>(&SEND_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, id, packet, data) };
    }

    unsafe extern "C" fn receive(this: *mut c_void, out: *mut MsvcString, timepoint: *const c_void) -> i32 {
        // SAFETY: the hook on the receive is installed, so its original slot holds a `ReceiveFn`.
        let original = unsafe { ScreenDetours::original::<ReceiveFn>(&RECEIVE_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let status = unsafe { original(this, out, timepoint) };
        if status == Self::HAS_DATA && Self::flags() & Self::LISTENER != 0 {
            Boundary::run((), || {
                WireAdapter::with((), |adapter| adapter.inbound(this.expose_provenance(), out.expose_provenance()));
            });
        }
        status
    }
}
