use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{
    CaretMeasureDataLayout, ClientInstance, MinecraftUiRenderContextLayout, RawObject, RectangleAreaLayout,
    StringViewLayout, TextMeasureDataLayout, UiColorLayout,
};
use crate::{EntryId, GameEntries};

/// A `MinecraftUIRenderContext*` of the game: the context that `ScreenView::render` draws with. The draw calls go
/// through its vtable; the fills draw at once and the text is queued until `flush_text`. Client main thread, inside
/// the render call; the calls are not thread-checked.
///
/// Every call gives `None` when its entry is off.
pub struct UiRenderContext<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> UiRenderContext<'a> {
    /// The feature of the UI render point and the draw calls.
    pub const FEATURE: &'static str = "game.ui_render";

    /// The entries of `FEATURE`: the render function, the pointer offset of the view, the screen context offset and
    /// the four draw slots.
    pub const ENTRIES: [EntryId; 7] = [
        EntryId::SCREEN_VIEW_RENDER,
        EntryId::SCREEN_VIEW_POINTER_LOCATION_PREVIOUS,
        EntryId::MINECRAFT_UI_RENDER_CONTEXT_M_SCREEN_CONTEXT,
        EntryId::MINECRAFT_UI_RENDER_CONTEXT_FILL_RECTANGLE,
        EntryId::MINECRAFT_UI_RENDER_CONTEXT_DRAW_RECTANGLE,
        EntryId::MINECRAFT_UI_RENDER_CONTEXT_DRAW_TEXT,
        EntryId::MINECRAFT_UI_RENDER_CONTEXT_FLUSH_TEXT,
    ];

    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `MinecraftUIRenderContext` of the game build `entries` was resolved for, and
    /// must stay live while the wrapper is used (the `UIRenderContext&` that `ScreenView::render` gets lives for
    /// that call).
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    // The entries the wrapper was made with.
    pub(crate) fn entries(&self) -> &'a GameEntries {
        self.entries
    }

    // The function in the vtable slot that `id` names.
    #[allow(unsafe_code)]
    fn function(&self, id: EntryId) -> Option<*const c_void> {
        let slot = self.entries.get(id)?;
        // SAFETY: `from_raw` guarantees a live context, whose first qword is its vtable, and `id` is a slot entry of
        // the MinecraftUIRenderContext class.
        unsafe { RawObject::slot(self.ptr.as_ptr(), slot) }
    }

    /// The client the context draws for (`mClient`). `None` when the entries are off or the client is null.
    #[allow(unsafe_code)]
    pub fn client(&self) -> Option<ClientInstance<'a>> {
        self.entries.get(EntryId::MINECRAFT_UI_RENDER_CONTEXT_M_SCREEN_CONTEXT)?;
        // SAFETY: `from_raw` guarantees a live context, which the prefix mirror fits.
        let client = unsafe { self.ptr.as_ptr().cast::<MinecraftUiRenderContextLayout>().read() }.client;
        // SAFETY: a live context holds a live client, or null.
        unsafe { ClientInstance::from_raw(client, self.entries) }
    }

    /// The `ScreenContext*` of the context (`mScreenContext`). `None` when the entry is off or the pointer is null.
    #[allow(unsafe_code)]
    pub fn screen_context(&self) -> Option<*mut c_void> {
        let offset = self.entries.get(EntryId::MINECRAFT_UI_RENDER_CONTEXT_M_SCREEN_CONTEXT)?;
        // SAFETY: `from_raw` guarantees a live context, and the entry is the offset of the pointer in it.
        unsafe { RawObject::pointer(self.ptr.as_ptr(), offset) }
    }

    /// Fills `area` with `color` at `alpha`, through `fillRectangle`. The fill draws at once.
    #[allow(unsafe_code)]
    pub fn fill_rect(&self, area: RectangleAreaLayout, color: UiColorLayout, alpha: f32) -> Option<()> {
        let function = self.function(EntryId::MINECRAFT_UI_RENDER_CONTEXT_FILL_RECTANGLE)?;
        // SAFETY: the slot holds `void fillRectangle(RectangleArea const&, mce::Color const&, float)`: this in rcx,
        // the area in rdx, the color in r8, the alpha in xmm3.
        let call: unsafe extern "C" fn(*mut c_void, *const RectangleAreaLayout, *const UiColorLayout, f32) =
            unsafe { std::mem::transmute(function) };
        // SAFETY: the context is live and the references live across the call.
        unsafe { call(self.ptr.as_ptr(), &raw const area, &raw const color, alpha) };
        Some(())
    }

    /// Draws the outline of `area`, `thickness` wide, through `drawRectangle`. The outline draws at once.
    #[allow(unsafe_code)]
    pub fn draw_rect(&self, area: RectangleAreaLayout, color: UiColorLayout, alpha: f32, thickness: i32) -> Option<()> {
        let function = self.function(EntryId::MINECRAFT_UI_RENDER_CONTEXT_DRAW_RECTANGLE)?;
        // SAFETY: the slot holds `void drawRectangle(RectangleArea const&, mce::Color const&, float, int)`: this in
        // rcx, the area in rdx, the color in r8, the alpha in xmm3, the thickness on the stack.
        let call: unsafe extern "C" fn(*mut c_void, *const RectangleAreaLayout, *const UiColorLayout, f32, i32) =
            unsafe { std::mem::transmute(function) };
        // SAFETY: the context is live and the references live across the call.
        unsafe { call(self.ptr.as_ptr(), &raw const area, &raw const color, alpha, thickness) };
        Some(())
    }

    /// Queues `text` in `font` inside `area`, through the `std::string_view` form of `drawText`, with no caret. The
    /// text draws at the next [`UiRenderContext::flush_text`].
    ///
    /// # Safety
    /// `font` must be a live `Font*` of the game. The game keeps the view of `text` and not the bytes: they must stay
    /// alive and unmoved until `flush_text` has run.
    #[allow(unsafe_code, clippy::too_many_arguments)]
    pub unsafe fn draw_text(
        &self,
        font: *mut c_void,
        area: RectangleAreaLayout,
        text: &str,
        color: UiColorLayout,
        alpha: f32,
        alignment: u32,
        measure: TextMeasureDataLayout,
    ) -> Option<()> {
        let function = self.function(EntryId::MINECRAFT_UI_RENDER_CONTEXT_DRAW_TEXT)?;
        // SAFETY: the slot holds `void drawText(Font&, RectangleArea const&, std::string_view, mce::Color const&,
        // float, ui::TextAlignment, TextMeasureData const&, CaretMeasureData const&)`: this in rcx, the font in rdx,
        // the area in r8, a pointer to a copy of the 16-byte view in r9, then on the stack the color, the alpha, the
        // alignment, the measure data and the caret data.
        let call: unsafe extern "C" fn(
            *mut c_void,
            *mut c_void,
            *const RectangleAreaLayout,
            *const StringViewLayout,
            *const UiColorLayout,
            f32,
            u32,
            *const TextMeasureDataLayout,
            *const CaretMeasureDataLayout,
        ) = unsafe { std::mem::transmute(function) };
        let view = StringViewLayout::of(text);
        let caret = CaretMeasureDataLayout::none();
        // SAFETY: the context and the font are live, the references live across the call, and the caller keeps the
        // text alive until the flush.
        unsafe {
            call(
                self.ptr.as_ptr(),
                font,
                &raw const area,
                &raw const view,
                &raw const color,
                alpha,
                alignment,
                &raw const measure,
                &raw const caret,
            );
        }
        Some(())
    }

    /// Draws the queued text, through `flushText` with no delta override. `delta_time` is the frame time in seconds.
    #[allow(unsafe_code)]
    pub fn flush_text(&self, delta_time: f32) -> Option<()> {
        let function = self.function(EntryId::MINECRAFT_UI_RENDER_CONTEXT_FLUSH_TEXT)?;
        // SAFETY: the slot holds `void flushText(float, std::optional<float>)`: this in rcx, the delta in xmm1, the
        // optional<float> by value in r8 (the value in the low 32 bits, the engaged flag in bit 32); 0 is empty.
        let call: unsafe extern "C" fn(*mut c_void, f32, u64) = unsafe { std::mem::transmute(function) };
        // SAFETY: the context is live.
        unsafe { call(self.ptr.as_ptr(), delta_time, 0) };
        Some(())
    }
}
