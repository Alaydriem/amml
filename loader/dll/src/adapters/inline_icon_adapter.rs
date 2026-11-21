use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlImageView, AmmlInlineIcon, AmmlStatus, AmmlThread};
use amml_data::{AdapterCatalog, Availability, NeedKind};
use amml_runtime::{IconKey, IconRefusal, InlineIconBook, TextureKey};

use crate::{CharWidthDetours, LoaderState, ObjectRead, TextCalls, TextureAdapter, TexturePump, ThreadNote, UiFlags};

static ADAPTER: OnceLock<InlineIconAdapter> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);

/// The inline chat icons (`ui.inline_icons`): small images that a chat line shows as one character of the default
/// font.
///
/// The icons live in the glyph page of the font, a 256 x 256 texture that the loader uploads under `font/glyph_E1`
/// (`InlineIconBook` keeps the pixels). Row 0 of the page is the vanilla row, shipped with the loader because the game
/// install has no `glyph_E1` file; every other row holds 16 icon cells. A change of the book queues an upload: the
/// texture pump runs `sync_page` at the next UI draw, which replaces the texture. The textures follow the rules of
/// `TextureBook`: the page is bound again after a world join, a dimension change and a resource pack change, and freed at
/// a world leave, which also frees every icon (the icons of a left world are `STALE`).
///
/// The width hook of the font (`CharWidthDetours`) gives a code point with an icon the advance of the icon. With no
/// icon registered it costs one relaxed load of `UiFlags`.
///
/// `register` runs on the game thread only, because it asks the client for the default font to prove that it is a
/// bitmap font. `update` and `release` run on any thread and only queue.
pub struct InlineIconAdapter {
    state: &'static LoaderState,
    calls: TextCalls,
    bitmap_vtable: usize,
    book: InlineIconBook,
    // The texture of the page in the texture book, when one was uploaded.
    page: Mutex<Option<TextureKey>>,
    // Orders a change of the book with the update of the `ICONS` flag.
    change: Mutex<()>,
    bitmap: AtomicBool,
    // Whether the page texture was uploaded and the game gave its data (`TextureBook::ready`). Until then no icon is
    // drawn: `UiFlags::ICONS` and `active` stay off. A later change of the page keeps it true, so the icons that are
    // already there do not flicker; a new icon shows after the next UI draw.
    page_ready: AtomicBool,
    // The page was queued for upload and `sync_page` has not looked at it since.
    recheck: AtomicBool,
    thread: ThreadNote,
    font_logged: AtomicBool,
}

impl InlineIconAdapter {
    pub const INLINE_ICONS: &'static str = "ui.inline_icons";
    const AREA_VERSION: u32 = 1;
    /// The name of the glyph page in the texture group.
    pub const PAGE_NAME: &'static str = "font/glyph_E1";
    // The owner of the page in the texture book: no mod has this index.
    const OWNER: usize = usize::MAX;
    // `mce::ImageFormat` of RGBA8, the only format an icon takes.
    const FORMAT_RGBA8: u32 = 4;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    // Row 0 of the glyph page: the vanilla row of the font, 16 pixel rows of 256 RGBA pixels.
    const ROW0: &'static [u8] = include_bytes!("../assets/glyph_e1_row0.rgba");

    /// Hooks the width of a `BitmapFont` character and offers `ui.inline_icons` when its catalog shape resolved, the
    /// texture adapter and the font layouts are there and the hook is in. Logs one line. Works once per process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&amml_data::EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(|row| row.id == Self::INLINE_ICONS).collect();
        if STARTED.swap(true, Ordering::AcqRel) {
            curia::warn!("the inline icon adapter is already installed");
            return rows;
        }
        let built = rows.iter().any(|row| row.shape.is_some()).then(|| Self::build(state)).flatten();
        let on = built.is_some() && CharWidthDetours::hook().install(state, Self::PRIORITY);
        if let (Some((calls, bitmap_vtable)), true) = (built, on) {
            Self::install_with(state, calls, bitmap_vtable);
        }
        for row in &mut rows {
            if row.shape.is_some() && ADAPTER.get().is_none() {
                row.shape = None;
                row.missing = vec!["a hook, a font layout or the texture adapter"];
            }
            match (row.shape, row.kind) {
                (Some(shape), NeedKind::Area) => curia::info!("area adapter", { area: row.id, shape: shape }),
                _ => curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    // The font calls and the `BitmapFont` vtable of the resolved entries; `None` without the texture adapter, a font
    // layout that fits or the vtable.
    fn build(state: &LoaderState) -> Option<(TextCalls, usize)> {
        TextureAdapter::get()?;
        let calls = TextCalls::new(state)?;
        let bitmap_vtable = state.resolve("BitmapFont.vtable").ok().and_then(|value| usize::try_from(value).ok())?;
        (bitmap_vtable != 0).then_some((calls, bitmap_vtable))
    }

    /// Installs the adapter over `calls` and the `BitmapFont` vtable at `bitmap_vtable`, and offers `ui.inline_icons`.
    /// It hooks nothing. False when an adapter is already installed.
    pub fn install_with(state: &'static LoaderState, calls: TextCalls, bitmap_vtable: usize) -> bool {
        let adapter = Self {
            state,
            calls,
            bitmap_vtable,
            book: InlineIconBook::new(Self::ROW0),
            page: Mutex::new(None),
            change: Mutex::new(()),
            bitmap: AtomicBool::new(false),
            page_ready: AtomicBool::new(false),
            recheck: AtomicBool::new(false),
            thread: ThreadNote::new(),
            font_logged: AtomicBool::new(false),
        };
        if ADAPTER.set(adapter).is_err() {
            return false;
        }
        state.areas().offer(Self::INLINE_ICONS, Self::AREA_VERSION);
        true
    }

    /// The installed adapter; `None` before `install`, and when inline icons are not served.
    pub fn get() -> Option<&'static InlineIconAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` when there is none.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&InlineIconAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// The icon book.
    pub fn book(&self) -> &InlineIconBook {
        &self.book
    }

    fn check_area(&self) -> Result<(), AmmlStatus> {
        if self.state.areas().is_available(Self::INLINE_ICONS, Self::AREA_VERSION) {
            Ok(())
        } else {
            Err(AmmlStatus::NOT_AVAILABLE)
        }
    }

    fn status(refusal: IconRefusal) -> AmmlStatus {
        match refusal {
            IconRefusal::Refused => AmmlStatus::REFUSED,
            IconRefusal::Stale => AmmlStatus::STALE,
            IconRefusal::BadImage => AmmlStatus::INVALID_ARGUMENT,
        }
    }

    fn key_of(icon: AmmlInlineIcon) -> IconKey {
        IconKey { id: icon.id, generation: icon.generation, code_point: icon.glyph }
    }

    // The size and the pixels of `image`. `INVALID_ARGUMENT` for a null view, a format that is not RGBA8 and pixels
    // that are null with a length.
    //
    // Safety: a non-null `image` points to a view whose pixels are valid for its `len` bytes for the call.
    unsafe fn read_image<'a>(image: *const AmmlImageView) -> Result<(u32, u32, &'a [u8]), AmmlStatus> {
        if image.is_null() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        // SAFETY: `image` is non-null and the caller promises it is readable.
        let view = unsafe { image.read() };
        if view.format != Self::FORMAT_RGBA8 || (view.pixels.is_null() && view.len != 0) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let pixels: &[u8] = if view.len == 0 {
            &[]
        } else {
            // SAFETY: the caller promises the pixels are valid for `len` bytes for the call.
            unsafe { std::slice::from_raw_parts(view.pixels, view.len) }
        };
        Ok((view.width, view.height, pixels))
    }

    // Sets the flag of the width hook from the book, and tells the pump that the page changed. Called after every change
    // of the book.
    fn changed(&self) {
        let _order = Self::lock(&self.change);
        self.set_flag();
        TexturePump::request();
    }

    // The flag of the width hook: an icon exists and its page is ready.
    fn set_flag(&self) {
        UiFlags::put(UiFlags::ICONS, self.book.has_icons() && self.page_ready.load(Ordering::Acquire));
    }

    // Sets the page state and the flag together.
    fn set_page_ready(&self, ready: bool) {
        let _order = Self::lock(&self.change);
        self.page_ready.store(ready, Ordering::Release);
        self.set_flag();
    }

    fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Registers the image of `owner` as an icon `advance` GUI units wide. Game thread only (`REFUSED` elsewhere);
    /// `NOT_AVAILABLE` when the default font is not a bitmap font.
    ///
    /// # Safety
    /// A non-null `image` points to a view whose pixels are valid for its `len` bytes for the call.
    pub(crate) unsafe fn register(
        &self,
        owner: usize,
        image: *const AmmlImageView,
        advance: f32,
    ) -> Result<AmmlInlineIcon, AmmlStatus> {
        self.check_area()?;
        let game = self.state.events().gate().on_game_thread();
        if self.thread.changed(if game { AmmlThread::GAME } else { AmmlThread::RENDER }) {
            curia::info!("ui thread", { hook: Self::INLINE_ICONS, game: game });
        }
        // SAFETY: the caller's guarantee is passed on.
        let (width, height, pixels) = unsafe { Self::read_image(image) }?;
        if !game {
            return Err(AmmlStatus::REFUSED);
        }
        self.refresh_font(true);
        if !self.bitmap.load(Ordering::Acquire) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let key = self.book.register(owner, width, height, pixels, advance).map_err(Self::status)?;
        self.changed();
        Ok(AmmlInlineIcon { id: key.id, generation: key.generation, glyph: key.code_point })
    }

    /// Replaces the image of `icon` of `owner`. Any thread.
    ///
    /// # Safety
    /// A non-null `image` points to a view whose pixels are valid for its `len` bytes for the call.
    pub(crate) unsafe fn update(
        &self,
        owner: usize,
        icon: AmmlInlineIcon,
        image: *const AmmlImageView,
    ) -> Result<(), AmmlStatus> {
        self.check_area()?;
        // SAFETY: the caller's guarantee is passed on.
        let (width, height, pixels) = unsafe { Self::read_image(image) }?;
        self.book.update(owner, Self::key_of(icon), width, height, pixels).map_err(Self::status)?;
        self.changed();
        Ok(())
    }

    /// Frees `icon` of `owner`; the cell is blanked and the page uploaded again at the next UI draw. Any thread.
    pub(crate) fn release(&self, owner: usize, icon: AmmlInlineIcon) -> Result<(), AmmlStatus> {
        self.check_area()?;
        self.book.release(owner, Self::key_of(icon)).map_err(Self::status)?;
        self.changed();
        Ok(())
    }

    /// The mod at `owner` is disabled: its icons are freed.
    pub fn disable_owner(&self, owner: usize) {
        let freed = self.book.disable_owner(owner);
        if freed > 0 {
            self.changed();
            curia::info!("inline icons freed", { reason: "mod_disabled", count: freed });
        }
    }

    /// A world was left: every icon is freed and the old icons are stale. The page leaves the texture book with the
    /// rest of the textures, so the adapter forgets it.
    pub fn world_left(&self) {
        let freed = self.book.world_left();
        *Self::lock(&self.page) = None;
        self.recheck.store(false, Ordering::Release);
        self.set_page_ready(false);
        self.changed();
        curia::info!("inline icons freed", { reason: "world_left", count: freed });
    }

    /// Whether the default font draws inline icons now: the last font check found a bitmap font, the area is served and the
    /// page texture is ready.
    pub(crate) fn active(&self) -> bool {
        self.bitmap.load(Ordering::Acquire) && self.page_ready.load(Ordering::Acquire) && self.check_area().is_ok()
    }

    /// Whether the texture pump should run once more after this draw point, so that `sync_page` looks at a page that was
    /// just uploaded.
    pub fn needs_sync(&self) -> bool {
        self.recheck.load(Ordering::Acquire)
    }

    /// The width of the character `character` when it is an icon; `None` for any other character.
    pub fn width(&self, character: i32) -> Option<f32> {
        self.book.advance_of(u32::try_from(character).ok()?)
    }

    /// Asks the client for its default font and keeps whether it is a bitmap font. Does nothing off the game thread
    /// (`on_game_thread` false) and when no font is there: the last answer stays.
    pub(crate) fn refresh_font(&self, on_game_thread: bool) {
        if !on_game_thread {
            return;
        }
        let Ok(frame) = self.calls.open_frame(0) else {
            self.bitmap.store(false, Ordering::Release);
            return;
        };
        // SAFETY: the frame holds the font handle the game just filled; the default font is a `shared_ptr<Font>` of it
        // and its first word is the `Font*`.
        let font = unsafe { ObjectRead::pointer(frame.address(), self.calls.default_font) };
        // SAFETY: a non-null default font is a live `Font` held by the handle; its first word is its vtable.
        let vtable = font.and_then(|font| unsafe { ObjectRead::pointer(font, 0) });
        let bitmap = vtable == Some(self.bitmap_vtable);
        drop(frame);
        self.bitmap.store(bitmap, Ordering::Release);
        if !self.font_logged.swap(true, Ordering::Relaxed) {
            curia::info!("inline icon font", { bitmap: bitmap });
        }
    }

    /// At a UI draw point: sends the glyph page to the game when the book changed it, and opens the gate of the icons once
    /// the page is ready.
    ///
    /// The first upload makes the texture under `font/glyph_E1`. Every later change replaces its pixels in place
    /// (`TextureBook::replace_pixels`): the page is never unloaded and uploaded under the same name, because the font may
    /// hold the texture it resolved. The page stays when the last icon is freed (its cells are blank). A page the game side
    /// freed (a world leave, a render device reset) is uploaded again when an icon needs it, and the gate is closed until
    /// that upload is ready.
    pub fn sync_page(&self) {
        let Some(textures) = TextureAdapter::get() else { return };
        let book = textures.book();
        let mut slot = Self::lock(&self.page);
        self.recheck.store(false, Ordering::Release);
        let current = slot.filter(|key| book.is_current(*key));
        if current.is_none() && slot.take().is_some() {
            self.set_page_ready(false);
        }
        if current.is_none() && self.book.is_empty() {
            // No page and no icon: nothing to send.
            self.book.take_page();
            return;
        }
        let changed = match (self.book.take_page(), current) {
            (Some(pixels), _) => Some(pixels),
            (None, Some(_)) => None,
            (None, None) => {
                // The page texture is gone: send the page again.
                self.book.mark_page_dirty();
                self.book.take_page()
            }
        };
        let side = amml_runtime::GlyphPage::SIDE;
        match (changed, current) {
            (Some(pixels), Some(key)) => {
                if book.replace_pixels(Self::OWNER, key, side, side, &pixels).is_err() {
                    curia::warn!("the glyph page was not queued for upload");
                }
                self.recheck.store(true, Ordering::Release);
            }
            (Some(pixels), None) => {
                *slot = book.upload_named(Self::OWNER, Self::PAGE_NAME, side, side, &pixels).ok();
                if slot.is_none() {
                    curia::warn!("the glyph page was not queued for upload");
                }
                self.recheck.store(true, Ordering::Release);
            }
            (None, Some(key)) => {
                if !self.page_ready.load(Ordering::Acquire) && book.ready(key) {
                    self.set_page_ready(true);
                    curia::info!("inline icon page ready", { icons: self.book.len() });
                }
            }
            (None, None) => {}
        }
    }
}
