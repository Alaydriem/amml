use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{NetworkConnectionLayout, NetworkIdentifierLayout, RawObject};
use crate::{EntryId, GameEntries};

/// A `NetworkConnection*` of the game.
pub struct NetworkConnection<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> NetworkConnection<'a> {
    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `NetworkConnection` of the game build `entries` was resolved for, and must stay
    /// live while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// The `mId` at the start of the connection.
    #[allow(unsafe_code)]
    pub fn id(&self) -> &NetworkIdentifierLayout {
        // SAFETY: `from_raw` guarantees a live connection, which starts with its `NetworkIdentifier`.
        unsafe { &(*self.ptr.as_ptr().cast::<NetworkConnectionLayout>()).id }
    }

    /// The `NetworkPeer*` in `mPeer`. `None` when the entry is off or the pointer is null. Game thread; the read is
    /// not thread-checked.
    #[allow(unsafe_code)]
    pub fn peer(&self) -> Option<*mut c_void> {
        let offset = self.entries.get(EntryId::NETWORK_CONNECTION_PEER)?;
        // SAFETY: `from_raw` guarantees a live connection, and the entry is the offset of its `mPeer` pointer.
        unsafe { RawObject::pointer(self.ptr.as_ptr(), offset) }
    }

    /// The code in slot `NetworkPeer.receivePacket` of the peer's vtable. `None` when an entry is off, the peer is
    /// null or the slot is null. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn peer_receive_slot(&self) -> Option<*const c_void> {
        let slot = self.entries.get(EntryId::NETWORK_PEER_RECEIVE_PACKET)?;
        let peer = self.peer()?;
        // SAFETY: a non-null `mPeer` of a live connection points to a `NetworkPeer`, whose first qword is its
        // vtable, and the entry is the slot of `receivePacket` in it.
        unsafe { RawObject::slot(peer, slot) }
    }
}
