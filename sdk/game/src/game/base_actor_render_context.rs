use std::ffi::c_void;

use crate::game::{BaseActorRenderContextLayout, ItemRenderer, UiRenderContext};
use crate::{EntryId, GameEntries};

/// A `BaseActorRenderContext` that the mod builds for item drawing, as the game's own item helper builds it: a
/// buffer of the game's size, filled by the constructor and destroyed by the destructor when dropped. Client main
/// thread, inside the render call.
pub struct BaseActorRenderContext<'a> {
    // 16-byte aligned words, so the buffer meets the object's alignment
    buffer: Vec<u128>,
    destroy: usize,
    entries: &'a GameEntries,
}

impl<'a> BaseActorRenderContext<'a> {
    // The largest object accepted: a size above this is a wrong entry, not a game class.
    const MAX_SIZE: usize = 0x1000;

    /// Builds a context for `screen_context`, `client` and `game` by calling the game's constructor. `None` when an
    /// entry is off, a pointer is null, or the `BaseActorRenderContext.size` entry is below the item renderer
    /// field or absurd.
    ///
    /// # Safety
    /// The three pointers must be a live `ScreenContext`, `IClientInstance` and `IMinecraftGame` of the game build
    /// `entries` was resolved for, and must stay live while the context is used.
    #[allow(unsafe_code)]
    pub unsafe fn new(
        entries: &'a GameEntries,
        screen_context: *mut c_void,
        client: *mut c_void,
        game: *mut c_void,
    ) -> Option<Self> {
        let size = usize::try_from(entries.get(EntryId::BASE_ACTOR_RENDER_CONTEXT_SIZE)?).ok()?;
        let ctor = usize::try_from(entries.get(EntryId::BASE_ACTOR_RENDER_CONTEXT_CTOR)?).ok()?;
        let destroy = usize::try_from(entries.get(EntryId::BASE_ACTOR_RENDER_CONTEXT_DTOR)?).ok()?;
        if !(BaseActorRenderContextLayout::MIN_SIZE..=Self::MAX_SIZE).contains(&size)
            || ctor == 0
            || destroy == 0
            || screen_context.is_null()
            || client.is_null()
            || game.is_null()
        {
            return None;
        }
        let mut buffer = vec![0u128; size.div_ceil(16)];
        // SAFETY: the entry is the address of `BaseActorRenderContext(ScreenContext&, IClientInstance&,
        // IMinecraftGame&)`: this in rcx, the references in rdx, r8 and r9, this returned in rax.
        let ctor: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
            unsafe { std::mem::transmute(ctor as *const c_void) };
        // SAFETY: the buffer is the size of the object and 16-byte aligned; the caller guarantees the references.
        unsafe { ctor(buffer.as_mut_ptr().cast(), screen_context, client, game) };
        Some(Self { buffer, destroy, entries })
    }

    /// Builds a context for the screen context and the client of `ui` and the game of that client. `None` as for
    /// [`BaseActorRenderContext::new`], or when the client's game is null.
    #[allow(unsafe_code)]
    pub fn for_ui(ui: &UiRenderContext<'a>) -> Option<Self> {
        let client = ui.client()?;
        // SAFETY: the render context is live, so are its screen context, its client and the client's game.
        unsafe { Self::new(ui.entries(), ui.screen_context()?, client.as_ptr(), client.minecraft_game()?) }
    }

    /// The pointer to the object, for the item renderer call.
    pub fn as_ptr(&self) -> *mut c_void {
        self.buffer.as_ptr().cast_mut().cast()
    }

    /// The item renderer the context holds (`mItemRenderer`); `None` when it is null.
    #[allow(unsafe_code)]
    pub fn item_renderer(&self) -> Option<ItemRenderer<'a>> {
        // SAFETY: the buffer holds a constructed object, which the prefix mirror fits (`new` checked the size).
        let renderer = unsafe { self.buffer.as_ptr().cast::<BaseActorRenderContextLayout>().read() }.item_renderer;
        // SAFETY: a constructed context holds a live ItemRenderer, or null.
        unsafe { ItemRenderer::from_raw(renderer, self.entries) }
    }
}

impl Drop for BaseActorRenderContext<'_> {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: the entry is the address of the complete destructor, which takes this in rcx.
        let destroy: unsafe extern "C" fn(*mut c_void) = unsafe { std::mem::transmute(self.destroy as *const c_void) };
        // SAFETY: the buffer holds the object `new` constructed, destroyed exactly once.
        unsafe { destroy(self.buffer.as_mut_ptr().cast()) };
    }
}
