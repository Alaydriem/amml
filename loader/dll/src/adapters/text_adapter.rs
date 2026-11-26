use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlColor, AmmlRect, AmmlStatus, AmmlStr, AmmlTextAlign, AmmlThread};
use amml_data::{AdapterCatalog, Availability, NeedKind};

use crate::{DrawCalls, FontFrame, LoaderState, TextCalls, ThreadNote, UiScope};

static ADAPTER: OnceLock<TextAdapter> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);

/// `int Font::getLineLength(std::string_view, float fontSize, bool showColorSymbol)`: rcx the font, rdx a pointer to
/// the 16-byte view, the size in xmm2, the bool in r9b.
type LineLengthFn = unsafe extern "C" fn(usize, *const [usize; 2], f32, u8) -> i32;
/// `void drawText(Font&, RectangleArea const&, std::string_view, mce::Color const&, float alpha, ui::TextAlignment,
/// TextMeasureData const&, CaretMeasureData const&)`.
type DrawTextFn = unsafe extern "C" fn(
    usize,
    usize,
    *const [f32; 4],
    *const [usize; 2],
    *const [f32; 4],
    f32,
    u32,
    *const [u32; 4],
    *const [u32; 2],
);
/// `void flushText(float deltaTime, std::optional<float> const& maxLifetime)`: the empty optional goes in r8 by value.
type FlushTextFn = unsafe extern "C" fn(usize, f32, u64);

/// The text calls of a `ScreenRender` listener (`ui.text`): the width of a text, and a text queued and drawn after
/// the last listener.
///
/// Every call needs the running `ScreenRender` scope of the view on this thread (`REFUSED` otherwise) and the game
/// thread (`REFUSED` otherwise): the font handle comes from the client. A scope opens one `FontHandle` at its first
/// text call and keeps it, with a copy of each queued text, until `finish`. The game keeps only the view of a queued
/// text, so the copies stay until the one `flushText` that `finish` makes after the last listener. A text is drawn
/// with the left alignment of the game from a start the loader computes from the measured width, so the alignment
/// values of the game are not needed. The text is one line high, from the top of the rectangle.
pub struct TextAdapter {
    state: &'static LoaderState,
    calls: TextCalls,
    frames: Mutex<Vec<FontFrame>>,
    // Whether `frames` may hold a frame: one relaxed load tells `finish` it has nothing to do.
    open: AtomicBool,
    thread: ThreadNote,
}

impl TextAdapter {
    pub const TEXT: &'static str = "ui.text";
    const AREA_VERSION: u32 = 1;
    /// The longest text a call takes, in bytes.
    pub const MAX_BYTES: usize = 256;
    /// The most texts one dispatch queues.
    pub const MAX_TEXTS: usize = 1024;
    // The dispatches whose frames are open at once: a nested dispatch is one more.
    const MAX_FRAMES: usize = 8;
    // The font size the widths are measured at, and the texts are drawn at.
    const FONT_SIZE: f32 = 1.0;
    // The height of one line of the default font at the font size, in GUI units.
    const LINE_HEIGHT: f32 = 8.0;
    // The left alignment of the game: the only value the loader passes.
    const LEFT: u32 = 0;
    // `TextMeasureData`: the font size, no line padding, then the shadow flag byte, then the alignment word.
    const SHADOW: u32 = 1;

    /// Offers `ui.text` when its catalog shape resolved and the layouts fit. Logs one line. Works once per process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&amml_data::EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(|row| row.id == Self::TEXT).collect();
        let calls = rows.iter().any(|row| row.shape.is_some()).then(|| TextCalls::new(state)).flatten();
        for row in &mut rows {
            if row.shape.is_some() && calls.is_none() {
                row.shape = None;
                row.missing = vec!["a font layout that fits"];
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

    /// Installs the adapter over `calls` and offers `ui.text`. False when an adapter is already installed.
    pub fn install_with(state: &'static LoaderState, calls: TextCalls) -> bool {
        if STARTED.swap(true, Ordering::AcqRel) {
            curia::warn!("the text adapter is already installed");
            return false;
        }
        let adapter = Self {
            state,
            calls,
            frames: Mutex::new(Vec::new()),
            open: AtomicBool::new(false),
            thread: ThreadNote::new(),
        };
        if ADAPTER.set(adapter).is_err() {
            return false;
        }
        state.areas().offer(Self::TEXT, Self::AREA_VERSION);
        true
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static TextAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&TextAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn lock(&self) -> MutexGuard<'_, Vec<FontFrame>> {
        self.frames.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    // The render context of the running `ScreenRender` scope of `view`, when a text call may run: the adapter serves
    // the area (`NOT_AVAILABLE`), the game gate is open (`NOT_READY`), and the innermost running scope of this thread
    // is the dispatch of `view` (`REFUSED`).
    fn enter(&self, view: u64) -> Result<usize, AmmlStatus> {
        if !self.state.areas().is_available(Self::TEXT, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        self.state.events().gate().require_open()?;
        if !UiScope::runs_view(view) {
            return Err(AmmlStatus::REFUSED);
        }
        UiScope::context().ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    // The text of `text`: at most `MAX_BYTES` long (`REFUSED`), valid UTF-8 (`INVALID_ARGUMENT`). The length is
    // checked before any byte is read.
    //
    // Safety: `text` is valid for reads of its length for the call.
    unsafe fn read<'a>(text: AmmlStr) -> Result<&'a str, AmmlStatus> {
        if text.len > Self::MAX_BYTES {
            return Err(AmmlStatus::REFUSED);
        }
        // SAFETY: the caller's guarantee is passed on.
        unsafe { text.as_str() }
    }

    // Only the game thread may open a font handle. Logs the thread of the call when it is a new one.
    fn require_game_thread(&self) -> Result<(), AmmlStatus> {
        let game = self.state.events().gate().on_game_thread();
        let thread = if game { AmmlThread::GAME } else { AmmlThread::RENDER };
        if self.thread.changed(thread) {
            curia::info!("ui thread", { hook: Self::TEXT, game: game });
        }
        if game { Ok(()) } else { Err(AmmlStatus::REFUSED) }
    }

    /// The width of `text` in GUI units, in the font of the running dispatch of `view`.
    ///
    /// # Safety
    /// `text` is valid for reads of its length for the call.
    pub(crate) unsafe fn width(&self, view: u64, text: AmmlStr) -> Result<f32, AmmlStatus> {
        self.enter(view)?;
        // SAFETY: the caller's guarantee is passed on.
        let text = unsafe { Self::read(text) }?;
        self.require_game_thread()?;
        self.with_frame(|frame| self.line_width(frame.font, text))
    }

    /// Queues `text` in `rect` of the running dispatch of `view`; it is drawn at `finish`.
    ///
    /// # Safety
    /// `text` is valid for reads of its length for the call.
    pub(crate) unsafe fn draw(
        &self,
        view: u64,
        text: AmmlStr,
        rect: AmmlRect,
        color: AmmlColor,
        alpha: f32,
        align: AmmlTextAlign,
    ) -> Result<(), AmmlStatus> {
        let context = self.enter(view)?;
        // SAFETY: the caller's guarantee is passed on.
        let text = unsafe { Self::read(text) }?;
        let values = [rect.x, rect.y, rect.width, rect.height, color.r, color.g, color.b, alpha];
        if !values.iter().all(|value| value.is_finite()) || align.0 > AmmlTextAlign::RIGHT.0 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        self.require_game_thread()?;
        if text.is_empty() {
            return Ok(());
        }
        // SAFETY: `enter` proved `context` is the live render context of a `ScreenRender` call running now on this
        // thread; its vtable holds `drawText` at the slot of the entry.
        let function = unsafe { DrawCalls::slot(context, self.calls.draw_slot) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `drawText`, whose arguments are these.
        let function = unsafe { std::mem::transmute::<usize, DrawTextFn>(function) };
        self.with_frame(|frame| {
            if frame.texts.len() >= Self::MAX_TEXTS {
                return Err(AmmlStatus::REFUSED);
            }
            let width = self.line_width(frame.font, text)?;
            let start = match align {
                AmmlTextAlign::CENTER => rect.x + (rect.width - width) / 2.0,
                AmmlTextAlign::RIGHT => rect.x + rect.width - width,
                _ => rect.x,
            };
            if !start.is_finite() {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            let copy: Box<[u8]> = text.as_bytes().into();
            let view = [copy.as_ptr().expose_provenance(), copy.len()];
            let area = [start, start + width, rect.y, rect.y + Self::LINE_HEIGHT];
            let measure = [Self::FONT_SIZE.to_bits(), 0, Self::SHADOW, 0];
            // SAFETY: the context and the font are live; the arrays live across the call; the game keeps `view`, and
            // the copy it points to stays in the frame until after the flush.
            unsafe {
                function(
                    context,
                    frame.font,
                    &area,
                    &view,
                    &[color.r, color.g, color.b, 1.0],
                    alpha,
                    Self::LEFT,
                    &measure,
                    &[0, 0],
                );
            }
            frame.texts.push(copy);
            Ok(())
        })
    }

    // Runs `body` on the frame of the running dispatch, opened when it is not there yet.
    fn with_frame<R>(&self, body: impl FnOnce(&mut FontFrame) -> Result<R, AmmlStatus>) -> Result<R, AmmlStatus> {
        let serial = UiScope::serial();
        let mut frames = self.lock();
        let index = match frames.iter().position(|frame| frame.serial == serial) {
            Some(index) => index,
            None => {
                if frames.len() >= Self::MAX_FRAMES {
                    return Err(AmmlStatus::REFUSED);
                }
                let frame = self.calls.open_frame(serial)?;
                frames.push(frame);
                self.open.store(true, Ordering::Relaxed);
                frames.len() - 1
            }
        };
        body(&mut frames[index])
    }

    // The width of `text` in `font` in GUI units.
    fn line_width(&self, font: usize, text: &str) -> Result<f32, AmmlStatus> {
        // SAFETY: `font` is live while the frame that holds it is; its vtable holds `getLineLength` at the slot.
        let function =
            unsafe { DrawCalls::slot(font, self.calls.line_length_slot) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `getLineLength`, whose arguments are these.
        let function = unsafe { std::mem::transmute::<usize, LineLengthFn>(function) };
        let view = [text.as_ptr().expose_provenance(), text.len()];
        // SAFETY: the font is live and the view lives across the call.
        let width = unsafe { function(font, &view, Self::FONT_SIZE, 0) };
        if width < 0 { Err(AmmlStatus::NOT_AVAILABLE) } else { Ok(width as f32) }
    }

    /// After the last `ScreenRender` listener of the scope with render context `context`: draws the texts the
    /// listeners queued with one `flushText`, then destroys the font handle and frees the copies.
    pub fn finish(&self, context: usize) {
        if !self.open.load(Ordering::Relaxed) {
            return;
        }
        let serial = UiScope::serial();
        let frame = {
            let mut frames = self.lock();
            let Some(index) = frames.iter().position(|frame| frame.serial == serial) else { return };
            let frame = frames.remove(index);
            self.open.store(!frames.is_empty(), Ordering::Relaxed);
            frame
        };
        if !frame.texts.is_empty() {
            // SAFETY: `context` is the live render context of the running scope; its vtable holds `flushText` at the
            // slot.
            if let Some(function) = unsafe { DrawCalls::slot(context, self.calls.flush_slot) } {
                // SAFETY: the slot holds `flushText`, whose arguments are these.
                let function = unsafe { std::mem::transmute::<usize, FlushTextFn>(function) };
                // SAFETY: the context is live; the font and the text copies live until `frame` drops below.
                unsafe { function(context, 0.0, 0) };
            }
        }
        drop(frame);
    }
}
