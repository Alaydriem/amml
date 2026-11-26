use std::ffi::c_void;
use std::mem::offset_of;

/// Layout mirror of the game's `brstd::flat_map<not_null<Actor*>, ShadowContext>` (48 bytes) that `LevelRendererCamera`
/// holds in `mActorRenderQueue`: two MSVC vectors side by side, the keys 8 bytes each and the values 1 byte each. The
/// same on 26.63 and 70.26 (survey section 8, row B1: keys `[rdi+1D8h]..[rdi+1E0h]`, values `[rdi+1F0h]..[rdi+1F8h]`,
/// the erase stores at +1E0h and +1F8h). The pointers belong to the game allocator: the loader never writes a `begin` or
/// a `cap` and never allocates.
#[repr(C)]
pub struct ActorRenderQueueLayout {
    keys_begin: *mut *mut c_void,
    keys_end: *mut *mut c_void,
    keys_cap: *mut *mut c_void,
    values_begin: *mut u8,
    values_end: *mut u8,
    values_cap: *mut u8,
}

const _: () = assert!(offset_of!(ActorRenderQueueLayout, keys_end) == 0x08);
const _: () = assert!(offset_of!(ActorRenderQueueLayout, keys_cap) == 0x10);
const _: () = assert!(offset_of!(ActorRenderQueueLayout, values_begin) == 0x18);
const _: () = assert!(offset_of!(ActorRenderQueueLayout, values_end) == 0x20);
const _: () = assert!(offset_of!(ActorRenderQueueLayout, values_cap) == 0x28);
const _: () = assert!(size_of::<ActorRenderQueueLayout>() == 0x30);

impl ActorRenderQueueLayout {
    /// Removes the pair whose key is `actor` from the queue at `queue`: the tail of the keys and the tail of the
    /// values move down by one element, and both `end` pointers go back by one element. Gives true when it removed a
    /// pair. Nothing changes, and it gives false, when `actor` is absent, a begin pointer is null or misaligned, an end
    /// is before its begin, or the keys and the values differ in length. The game's own override erases from the same
    /// queue the same way.
    ///
    /// # Safety
    /// `queue` is the address of a live `mActorRenderQueue`, and the call runs inside a `queueRenderEntities` detour
    /// after the original returned, on the thread that runs it, so that nothing else reads or writes the queue.
    pub unsafe fn erase_actor(queue: usize, actor: usize) -> bool {
        let layout = std::ptr::with_exposed_provenance_mut::<ActorRenderQueueLayout>(queue);
        // SAFETY: the caller guarantees a live queue; the fields are read by value.
        let (keys_begin, keys_end, values_begin, values_end) =
            unsafe { ((*layout).keys_begin, (*layout).keys_end, (*layout).values_begin, (*layout).values_end) };
        if keys_begin.is_null() || values_begin.is_null() || keys_end < keys_begin || values_end < values_begin {
            return false;
        }
        // The byte distances, so a malformed vector is refused before any pointer arithmetic.
        let keys_bytes = keys_end.addr() - keys_begin.addr();
        let values_len = values_end.addr() - values_begin.addr();
        if !keys_bytes.is_multiple_of(size_of::<*mut c_void>())
            || !keys_begin.addr().is_multiple_of(align_of::<usize>())
        {
            return false;
        }
        let keys_len = keys_bytes / size_of::<*mut c_void>();
        if keys_len != values_len {
            return false;
        }
        // SAFETY: `keys_len` elements are live and readable from `keys_begin`.
        let keys = unsafe { std::slice::from_raw_parts(keys_begin, keys_len) };
        let Some(index) = keys.iter().position(|key| key.addr() == actor) else {
            return false;
        };
        let tail = keys_len - index - 1;
        // SAFETY: the move stays inside the live keys: `tail` elements from `index + 1` down to `index`; `copy`
        // handles the overlap.
        unsafe { std::ptr::copy(keys_begin.add(index + 1), keys_begin.add(index), tail) };
        // SAFETY: as above, for the values, which have the same length.
        unsafe { std::ptr::copy(values_begin.add(index + 1), values_begin.add(index), tail) };
        // SAFETY: the vectors hold at least one element, so one element back is inside the buffer; the queue is live.
        unsafe {
            (*layout).keys_end = keys_end.sub(1);
            (*layout).values_end = values_end.sub(1);
        }
        true
    }
}
