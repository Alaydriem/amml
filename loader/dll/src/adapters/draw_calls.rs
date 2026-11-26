use crate::{LoaderState, ObjectRead};

/// The slots, offsets and functions the draw calls of the render context use.
///
/// `*_slot` are slots of the `MinecraftUIRenderContext` vtable. `screen_context` is the offset of the
/// `ScreenContext*` in the context, `gui_data` the offset of the raw `GuiData*` in the `ScreenContext`, and
/// `gui_scale` the address of `float GuiData::getGuiScale() const`. `queued_images` is the offset of the begin and
/// end pointers of the image queue of the context, when the build has the entry.
#[derive(Clone, Copy, Debug)]
pub struct DrawCalls {
    pub fill_slot: usize,
    pub outline_slot: usize,
    pub image_slot: usize,
    pub flush_slot: usize,
    pub screen_context: usize,
    pub gui_data: usize,
    pub gui_scale: usize,
    pub queued_images: Option<usize>,
}

impl DrawCalls {
    /// The calls from the resolved entries; `None` when one that is not optional is missing.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        Some(Self {
            fill_slot: value("MinecraftUIRenderContext.fillRectangle")?,
            outline_slot: value("MinecraftUIRenderContext.drawRectangle")?,
            image_slot: value("MinecraftUIRenderContext.drawImage")?,
            flush_slot: value("MinecraftUIRenderContext.flushImages")?,
            screen_context: value("MinecraftUIRenderContext.mScreenContext")?,
            gui_data: value("ScreenContext.guiData")?,
            gui_scale: value("GuiData.getGuiScale").filter(|address| *address != 0)?,
            queued_images: value("MinecraftUIRenderContext.mQueuedImages"),
        })
    }

    /// The function in `slot` of the vtable of the object at `object`; `None` for a null object, vtable or
    /// function.
    ///
    /// # Safety
    /// `object` is a live game object whose first word is its vtable, and `slot` is inside that vtable.
    pub unsafe fn slot(object: usize, slot: usize) -> Option<usize> {
        // SAFETY: the caller guarantees the first word of `object` is its vtable pointer.
        let vtable = unsafe { ObjectRead::pointer(object, 0) }?;
        // SAFETY: the caller guarantees `slot` is inside the vtable.
        unsafe { ObjectRead::pointer(vtable, slot.checked_mul(std::mem::size_of::<usize>())?) }
    }
}
