use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{NetworkConnection, NetworkIdentifierEquality, NetworkIdentifierLayout, RawObject};
use crate::{EntryId, GameEntries};

/// A `NetworkSystem*` of the game.
pub struct NetworkSystem<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> NetworkSystem<'a> {
    // The size of one element of `mConnections`: a `unique_ptr<NetworkConnection>`.
    const ELEMENT_SIZE: usize = 8;

    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `NetworkSystem` of the game build `entries` was resolved for, and must stay
    /// live while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// The first connection whose `mId` equals `id` under the game's per-type compare. Null elements are skipped.
    /// `None` when the entry is off, the vector is empty or malformed, or no connection matches. Game thread; the
    /// read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn connection_for(&self, id: &NetworkIdentifierLayout) -> Option<NetworkConnection<'a>> {
        let offset = self.entries.get(EntryId::NETWORK_SYSTEM_CONNECTIONS)?;
        // SAFETY: `from_raw` guarantees a live system, and the entry is the offset of the `mConnections` vector,
        // whose begin and end pointers are its first two qwords.
        let begin = unsafe { RawObject::read::<usize>(self.ptr.as_ptr(), offset) }?;
        // SAFETY: as above, for the end pointer.
        let end = unsafe { RawObject::read::<usize>(self.ptr.as_ptr(), offset.checked_add(8)?) }?;
        if begin == 0 || end < begin {
            return None;
        }
        let equality = NetworkIdentifierEquality::new();
        for index in 0..(end - begin) / Self::ELEMENT_SIZE {
            let element = (begin + index * Self::ELEMENT_SIZE) as *const *mut c_void;
            // SAFETY: the index is inside the vector the game holds, and each element is a connection pointer.
            let pointer = unsafe { element.read_unaligned() };
            // SAFETY: a non-null element is a live connection owned by the vector.
            let Some(connection) = (unsafe { NetworkConnection::from_raw(pointer, self.entries) }) else {
                continue;
            };
            if equality.equals(connection.id(), id) {
                return Some(connection);
            }
        }
        None
    }

    /// Whether this is a server, through `NetworkSystem::isServer`. `None` when the entry is off. Game thread; the
    /// call is not thread-checked.
    #[allow(unsafe_code)]
    pub fn is_server(&self) -> Option<bool> {
        let function = usize::try_from(self.entries.get(EntryId::NETWORK_SYSTEM_IS_SERVER)?).ok()? as *const c_void;
        // SAFETY: the entry is the address of `bool NetworkSystem::isServer() const`.
        Some(unsafe { RawObject::call_bool(function, self.ptr.as_ptr()) })
    }
}
