use std::ffi::c_void;

use crate::game::ActorRenderQueueLayout;
use crate::{EntryId, GameEntries};

/// The `brstd::flat_map<not_null<Actor*>, ShadowContext>` that `LevelRendererCamera` holds in `mActorRenderQueue`.
pub struct ActorRenderQueue<'a> {
    layout: &'a mut ActorRenderQueueLayout,
}

impl<'a> ActorRenderQueue<'a> {
    pub fn from_layout(layout: &'a mut ActorRenderQueueLayout) -> Self {
        Self { layout }
    }

    /// The queue inside a `LevelRendererCamera`. `None` when the entry is off or `renderer` is null.
    ///
    /// # Safety
    /// `renderer` must be null or a live `LevelRendererCamera` of the game build `entries` was resolved for, and
    /// must stay live, with nothing else using the queue, while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_renderer(renderer: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        let offset = usize::try_from(entries.get(EntryId::LEVEL_RENDERER_CAMERA_ACTOR_RENDER_QUEUE)?).ok()?;
        if renderer.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees a live renderer, which holds the queue by value at the entry's offset.
        let layout = unsafe { &mut *renderer.cast::<u8>().add(offset).cast::<ActorRenderQueueLayout>() };
        Some(Self { layout })
    }

    /// Removes the pair whose key is `actor`: the tail of the keys and the tail of the values move down by one
    /// element, and both `end` pointers go back by one element. Returns `true` when it removed a pair. Nothing
    /// changes, and it returns `false`, when `actor` is absent, a pointer is null or reversed, or the keys and the
    /// values differ in length. It never writes `begin` or `cap` and never allocates.
    ///
    /// # Safety
    /// Call it only inside a `LevelRendererPlayer.queueRenderEntities` detour, after the original returns, on the
    /// render thread. The game's own override erases from the same queue the same way, so this is a state the game
    /// makes itself. The buffers must be live and owned by the game, with no other reader or writer at this time.
    #[allow(unsafe_code)]
    pub unsafe fn erase_actor(&mut self, actor: *mut c_void) -> bool {
        let layout = &mut *self.layout;
        if layout.keys_begin.is_null() || layout.values_begin.is_null() {
            return false;
        }
        if layout.keys_end < layout.keys_begin || layout.values_end < layout.values_begin {
            return false;
        }
        // the byte distances, so a malformed vector is refused before any pointer arithmetic
        let keys_bytes = layout.keys_end as usize - layout.keys_begin as usize;
        let values_len = layout.values_end as usize - layout.values_begin as usize;
        if !keys_bytes.is_multiple_of(size_of::<*mut c_void>()) {
            return false;
        }
        if !(layout.keys_begin as usize).is_multiple_of(align_of::<*mut c_void>()) {
            return false;
        }
        let keys_len = keys_bytes / size_of::<*mut c_void>();
        if keys_len != values_len {
            return false;
        }
        // SAFETY: `keys_len` elements are live and readable from `keys_begin`.
        let keys = unsafe { std::slice::from_raw_parts(layout.keys_begin, keys_len) };
        let Some(index) = keys.iter().position(|key| *key == actor) else {
            return false;
        };
        let tail = keys_len - index - 1;
        // SAFETY: the move stays inside the live keys: `tail` elements from `index + 1` down to `index`. `copy`
        // handles the overlap.
        unsafe { std::ptr::copy(layout.keys_begin.add(index + 1), layout.keys_begin.add(index), tail) };
        // SAFETY: as above, for the values, which have the same length.
        unsafe { std::ptr::copy(layout.values_begin.add(index + 1), layout.values_begin.add(index), tail) };
        // SAFETY: the vectors hold at least one element, so one element back is inside the buffer.
        layout.keys_end = unsafe { layout.keys_end.sub(1) };
        // SAFETY: as above.
        layout.values_end = unsafe { layout.values_end.sub(1) };
        true
    }
}
