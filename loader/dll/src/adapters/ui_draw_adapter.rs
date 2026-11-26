use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlColor, AmmlRect, AmmlStatus, AmmlTextureId, AmmlUv};
use amml_data::{AdapterCatalog, Availability, NeedKind};
use amml_runtime::TextureKey;

use crate::{DrawBatch, DrawCalls, LoaderState, ObjectRead, TextureAdapter, UiScope};

static ADAPTER: OnceLock<UiDrawAdapter> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);

/// `void fillRectangle(RectangleArea const&, mce::Color const&, float)`: the area is x0, x1, y0, y1, the color r, g,
/// b, a; the alpha is in xmm3.
type FillFn = unsafe extern "C" fn(usize, *const [f32; 4], *const [f32; 4], f32);
/// `void drawRectangle(RectangleArea const&, mce::Color const&, float, int)`: the thickness is on the stack.
type OutlineFn = unsafe extern "C" fn(usize, *const [f32; 4], *const [f32; 4], f32, i32);
/// `void drawImage(BedrockTextureData const&, vec2 const& position, vec2 const& size, vec2 const& uv,
/// vec2 const& uv_size, bool)`.
type ImageFn =
    unsafe extern "C" fn(usize, usize, *const [f32; 2], *const [f32; 2], *const [f32; 2], *const [f32; 2], u8);
/// `void flushImages(mce::Color const&, float, HashedString const&)`.
type FlushFn = unsafe extern "C" fn(usize, *const [f32; 4], f32, *const c_void);
/// `float GuiData::getGuiScale() const`.
type GuiScaleFn = unsafe extern "C" fn(usize) -> f32;

/// The draw calls of a `ScreenRender` listener (`ui.draw`): fills and outlines at once, images queued and flushed.
///
/// Every call needs the running `ScreenRender` scope of the view on this thread (`REFUSED` otherwise); the draw goes
/// to the render context of that scope. The game draws a whole image queue with the texture of its last item, so
/// the adapter keeps the images of one texture and tint as one batch: it flushes the batch when an image of another
/// texture or tint comes, before a fill or an outline (so the order of the calls is the order on the screen), and
/// after the last listener (`finish`). The flush uses an empty material name, which gives the game's own textured
/// material. While the game itself left images queued, `can_draw` is false and an image is refused: the flush would
/// draw them with the wrong texture.
pub struct UiDrawAdapter {
    state: &'static LoaderState,
    calls: DrawCalls,
    // The address of a leaked, writable, empty `HashedString`: the game's has a mutable last-match member.
    material: usize,
    pending: Mutex<Option<DrawBatch>>,
}

impl UiDrawAdapter {
    pub const DRAW: &'static str = "ui.draw";
    const AREA_VERSION: u32 = 2;
    // Words of a leaked empty `HashedString`: zero, and the small-string capacity of 15 at the member `_Myres`.
    const MATERIAL_WORDS: usize = 8;
    const SMALL_STRING_CAPACITY: u64 = 15;
    const CAPACITY_WORD: usize = 4;

    /// Offers `ui.draw` when its catalog shape resolved. Logs one line. Works once per process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&amml_data::EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(|row| row.id == Self::DRAW).collect();
        let calls = rows.iter().any(|row| row.shape.is_some()).then(|| DrawCalls::new(state)).flatten();
        for row in &mut rows {
            if row.shape.is_some() && calls.is_none() {
                row.shape = None;
                row.missing = vec!["a draw call entry"];
            }
            match (row.shape, row.kind) {
                (Some(shape), NeedKind::Area) => {
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                _ => curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") }),
            }
        }
        if let Some(calls) = calls {
            Self::install_with(state, calls);
        }
        rows
    }

    /// Installs the adapter over `calls` and offers `ui.draw`. False when an adapter is already installed.
    pub fn install_with(state: &'static LoaderState, calls: DrawCalls) -> bool {
        if STARTED.swap(true, Ordering::AcqRel) {
            curia::warn!("the draw adapter is already installed");
            return false;
        }
        let mut words = Box::new([0u64; Self::MATERIAL_WORDS]);
        words[Self::CAPACITY_WORD] = Self::SMALL_STRING_CAPACITY;
        let material = Box::leak(words) as *mut [u64; Self::MATERIAL_WORDS] as usize;
        let adapter = Self { state, calls, material, pending: Mutex::new(None) };
        if ADAPTER.set(adapter).is_err() {
            return false;
        }
        state.areas().offer(Self::DRAW, Self::AREA_VERSION);
        true
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static UiDrawAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&UiDrawAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn lock(&self) -> MutexGuard<'_, Option<DrawBatch>> {
        self.pending.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    // The render context of the running `ScreenRender` scope of `view`, when a table call may draw: the adapter
    // serves the area (`NOT_AVAILABLE`), the game gate is open (`NOT_READY`), and the innermost running scope of this
    // thread is the dispatch of `view` (`REFUSED`).
    fn enter(&self, view: u64) -> Result<usize, AmmlStatus> {
        if !self.state.areas().is_available(Self::DRAW, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        self.state.events().gate().require_open()?;
        if !UiScope::runs_view(view) {
            return Err(AmmlStatus::REFUSED);
        }
        UiScope::context().ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    fn finite(values: &[f32]) -> Result<(), AmmlStatus> {
        if values.iter().all(|value| value.is_finite()) { Ok(()) } else { Err(AmmlStatus::INVALID_ARGUMENT) }
    }

    // `RectangleArea`: x0, x1, y0, y1.
    fn area(rect: AmmlRect) -> [f32; 4] {
        [rect.x, rect.x + rect.width, rect.y, rect.y + rect.height]
    }

    /// Fills `rect` of `view` at once.
    pub(crate) fn fill(&self, view: u64, rect: AmmlRect, color: AmmlColor, alpha: f32) -> Result<(), AmmlStatus> {
        let context = self.enter(view)?;
        Self::finite(&[rect.x, rect.y, rect.width, rect.height, color.r, color.g, color.b, alpha])?;
        self.flush(context);
        // SAFETY: `enter` proved `context` is the live render context of a `ScreenRender` call running now on this
        // thread; its vtable holds `fillRectangle` at the slot of the entry.
        let function = unsafe { DrawCalls::slot(context, self.calls.fill_slot) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `fillRectangle`, whose arguments are these.
        let function = unsafe { std::mem::transmute::<usize, FillFn>(function) };
        // SAFETY: the context is live; the area and the color live across the call.
        unsafe { function(context, &Self::area(rect), &[color.r, color.g, color.b, 1.0], alpha) };
        Ok(())
    }

    /// Draws the border of `rect` of `view` at once.
    pub(crate) fn outline(
        &self,
        view: u64,
        rect: AmmlRect,
        color: AmmlColor,
        alpha: f32,
        thickness: i32,
    ) -> Result<(), AmmlStatus> {
        let context = self.enter(view)?;
        Self::finite(&[rect.x, rect.y, rect.width, rect.height, color.r, color.g, color.b, alpha])?;
        if thickness < 1 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        self.flush(context);
        // SAFETY: as for `fill`, with the slot of `drawRectangle`.
        let function = unsafe { DrawCalls::slot(context, self.calls.outline_slot) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `drawRectangle`, whose arguments are these.
        let function = unsafe { std::mem::transmute::<usize, OutlineFn>(function) };
        // SAFETY: the context is live; the area and the color live across the call.
        unsafe { function(context, &Self::area(rect), &[color.r, color.g, color.b, 1.0], alpha, thickness) };
        Ok(())
    }

    /// Queues the part `uv` of `texture` into `rect` of `view`. A texture that is not ready is skipped.
    pub(crate) fn image(
        &self,
        view: u64,
        texture: AmmlTextureId,
        rect: AmmlRect,
        uv: AmmlUv,
        tint: AmmlColor,
        alpha: f32,
    ) -> Result<(), AmmlStatus> {
        let context = self.enter(view)?;
        Self::finite(&[
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            uv.u0,
            uv.v0,
            uv.u1,
            uv.v1,
            tint.r,
            tint.g,
            tint.b,
            alpha,
        ])?;
        if !self.may_queue(context) {
            return Err(AmmlStatus::REFUSED);
        }
        let key = TextureKey { id: texture.id, generation: texture.generation };
        let held = TextureAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.drawable(key))?;
        let Some(held) = held else { return Ok(()) };
        // SAFETY: `held` is the address of a `TexturePtr` the texture book holds; its first word is the pointer to
        // the `BedrockTextureData`, which the game keeps alive while the `TexturePtr` is held.
        let Some(data) = (unsafe { ObjectRead::pointer(held, 0) }) else { return Ok(()) };
        let batch = DrawBatch { texture: data, tint: [tint.r, tint.g, tint.b], alpha };
        if self.lock().is_some_and(|pending| pending != batch) {
            self.flush(context);
        }
        // SAFETY: as for `fill`, with the slot of `drawImage`.
        let function = unsafe { DrawCalls::slot(context, self.calls.image_slot) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `drawImage`, whose arguments are these.
        let function = unsafe { std::mem::transmute::<usize, ImageFn>(function) };
        // SAFETY: the context is live; the vectors live across the call (the game copies them into its queue item);
        // the texture data stays alive until the flush, which runs before the texture pump frees anything.
        unsafe {
            function(
                context,
                data,
                &[rect.x, rect.y],
                &[rect.width, rect.height],
                &[uv.u0, uv.v0],
                &[uv.u1 - uv.u0, uv.v1 - uv.v0],
                0,
            );
        }
        *self.lock() = Some(batch);
        Ok(())
    }

    /// The render context of the running `ScreenRender` scope of `view`, after the images the listeners queued are
    /// drawn: an item the game draws itself goes to the screen after the calls before it.
    pub(crate) fn before_item(&self, view: u64) -> Result<usize, AmmlStatus> {
        let context = self.enter(view)?;
        self.flush(context);
        Ok(context)
    }

    /// The GUI scale of `view`: screen pixels per GUI unit.
    pub(crate) fn gui_scale(&self, view: u64) -> Result<f32, AmmlStatus> {
        let context = self.enter(view)?;
        // SAFETY: `enter` proved `context` is the live render context; it holds its `ScreenContext*` at the offset.
        let screen =
            unsafe { ObjectRead::pointer(context, self.calls.screen_context) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the `ScreenContext` holds the raw `GuiData*` at the offset.
        let gui = unsafe { ObjectRead::pointer(screen, self.calls.gui_data) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the entry is the address of `float GuiData::getGuiScale() const`: this in rcx, the scale in xmm0.
        let function = unsafe { std::mem::transmute::<usize, GuiScaleFn>(self.calls.gui_scale) };
        // SAFETY: the `GuiData` is live.
        let scale = unsafe { function(gui) };
        if scale.is_finite() && scale > 0.0 { Ok(scale) } else { Err(AmmlStatus::NOT_AVAILABLE) }
    }

    /// Whether `view` may queue an image now.
    pub(crate) fn can_draw(&self, view: u64) -> bool {
        self.enter(view).is_ok_and(|context| self.may_queue(context))
    }

    // Whether an image may be queued in `context`: the loader has images queued already, or the game has none.
    fn may_queue(&self, context: usize) -> bool {
        if self.lock().is_some() {
            return true;
        }
        let Some(offset) = self.calls.queued_images else { return true };
        // SAFETY: a live context holds the begin and end pointers of its image queue at the entry's offset.
        let begin = unsafe { ObjectRead::value::<usize>(context, offset) };
        // SAFETY: as above, the end pointer follows the begin pointer.
        let end = unsafe { ObjectRead::value::<usize>(context, offset + std::mem::size_of::<usize>()) };
        begin == end
    }

    // Draws the queued batch, if any, with its texture and tint.
    fn flush(&self, context: usize) {
        let Some(batch) = self.lock().take() else { return };
        // SAFETY: `context` is the live render context of a running scope; its vtable holds `flushImages` at the slot.
        let Some(function) = (unsafe { DrawCalls::slot(context, self.calls.flush_slot) }) else { return };
        // SAFETY: the slot holds `flushImages`, whose arguments are these.
        let function = unsafe { std::mem::transmute::<usize, FlushFn>(function) };
        let color = [batch.tint[0], batch.tint[1], batch.tint[2], 1.0];
        // SAFETY: the context is live; the material is a leaked, writable, empty `HashedString`.
        unsafe { function(context, &color, batch.alpha, std::ptr::with_exposed_provenance::<c_void>(self.material)) };
    }

    /// After the last `ScreenRender` listener of a scope with render context `context`: draws the images still
    /// queued.
    pub fn finish(&self, context: usize) {
        self.flush(context);
    }
}
