use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{BaseActorRenderContext, ItemStackBaseRef};
use crate::{EntryId, GameEntries};

/// An `ItemRenderer*` of the game, as a `BaseActorRenderContext` holds it. Client main thread, inside the render
/// call; the call is not thread-checked.
pub struct ItemRenderer<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> ItemRenderer<'a> {
    /// The feature of the item drawing: the render function, the context constructor, destructor and size, the
    /// client's game, and the item calls of the stack wrapper.
    pub const FEATURE: &'static str = "game.item_draw";

    /// The entries of `FEATURE`.
    pub const ENTRIES: [EntryId; 10] = [
        EntryId::ITEM_RENDERER_RENDER_GUI_ITEM_NEW,
        EntryId::BASE_ACTOR_RENDER_CONTEXT_CTOR,
        EntryId::BASE_ACTOR_RENDER_CONTEXT_DTOR,
        EntryId::BASE_ACTOR_RENDER_CONTEXT_SIZE,
        EntryId::CLIENT_INSTANCE_GET_MINECRAFT_GAME,
        EntryId::ITEM_GET_ANIMATION_FRAME_FOR,
        EntryId::ITEM_IS_GLINT,
        EntryId::ITEM_GET_MAX_DAMAGE,
        EntryId::ITEM_STACK_BASE_GET_DAMAGE_VALUE,
        EntryId::ITEM_STACK_BASE_IS_DAMAGED,
    ];

    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `ItemRenderer` of the game build `entries` was resolved for, and must stay live
    /// while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// Draws the picture of `stack` with its top left corner at (`x`, `y`) in GUI units, through
    /// `ItemRenderer::renderGuiItemNew`: `frame` is the animation frame, `foil` draws the glint, `alpha` is the
    /// transparency, `scale` the size factor and `z_order` the layer. The light argument is 1.0, as the game's own
    /// item helper passes it. `None` when the entry is off.
    #[allow(unsafe_code, clippy::too_many_arguments)]
    pub fn render_gui_item(
        &self,
        context: &BaseActorRenderContext<'_>,
        stack: &ItemStackBaseRef<'_>,
        frame: i32,
        x: f32,
        y: f32,
        foil: bool,
        alpha: f32,
        scale: f32,
        z_order: i32,
    ) -> Option<()> {
        let function = usize::try_from(self.entries.get(EntryId::ITEM_RENDERER_RENDER_GUI_ITEM_NEW)?).ok()?;
        if function == 0 {
            return None;
        }
        // SAFETY: the entry is the address of `void ItemRenderer::renderGuiItemNew(BaseActorRenderContext&,
        // ItemStack const&, int frame, float x, float y, bool foil, float transparency, float light, float scale,
        // int zOrder)`: this in rcx, the context in rdx, the stack in r8, the frame in r9d, then on the stack x, y,
        // foil, transparency, light, scale and the z order.
        let call: unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_void, i32, f32, f32, u8, f32, f32, f32, i32) =
            unsafe { std::mem::transmute(function as *const c_void) };
        // SAFETY: the renderer, the context and the stack are live for the call.
        unsafe {
            call(
                self.ptr.as_ptr(),
                context.as_ptr(),
                stack.as_ptr(),
                frame,
                x,
                y,
                u8::from(foil),
                alpha,
                1.0,
                scale,
                z_order,
            );
        }
        Some(())
    }
}
