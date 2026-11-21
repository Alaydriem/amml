use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use amml_abi::{AmmlStatus, AmmlTextureId, AmmlTextureSource, AmmlThread};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::SharedPtr;
use amml_runtime::{
    TextureBook, TextureJob, TextureKey, TextureRefusal, TextureRelease, TextureSourceKind, TextureTarget,
};

use crate::{
    ClientAdapter, ImageDescriptionLayout, InlineIconAdapter, ItemDrawAdapter, LoaderState, ObjectRead, PathViewLayout,
    SessionAdapter, TextureCalls, TexturePtrLayout, TexturePump, TextureReloadAdapter, ThreadNote,
};

static ADAPTER: OnceLock<TextureAdapter> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);

/// `mce::TexturePtr MinecraftUIRenderContext::getTexture(ResourceLocation const&, bool)`: rdx the hidden result.
type ContextGetTextureFn = unsafe extern "C" fn(usize, *mut TexturePtrLayout, *const c_void, u8) -> *mut c_void;
/// `mce::TexturePtr TextureGroupBase::getTexture(ResourceLocation const&, bool, std::optional<uint>, layer)`.
type GroupGetTextureFn = unsafe extern "C" fn(usize, *mut TexturePtrLayout, *const c_void, u8, u64, u32) -> *mut c_void;
/// `ResourceLocation::ResourceLocation(Core::PathView)`.
type LocationCtorFn = unsafe extern "C" fn(*mut c_void, *const PathViewLayout) -> *mut c_void;
/// `ImageBuffer::ImageBuffer(ImageDescription const&)`.
type ImageBufferCtorFn = unsafe extern "C" fn(*mut u64, *const ImageDescriptionLayout) -> *mut u64;
/// The blob deleter of an `ImageBuffer`.
type BlobDeleterFn = unsafe extern "C" fn(*mut u8);
/// `BedrockTexture& TextureGroup::uploadTexture(ResourceLocation const&, cg::ImageBuffer)`: the buffer by address.
type UploadFn = unsafe extern "C" fn(usize, *const c_void, *mut u64) -> usize;
/// `bool TextureGroup::unloadTexture(ResourceLocation const&, bool)`.
type UnloadFn = unsafe extern "C" fn(usize, *const c_void, u8) -> u8;
/// `shared_ptr<TextureGroup> ClientInstance::getTextureGroup()`: rdx the hidden result.
type GetTextureGroupFn = unsafe extern "C" fn(usize, *mut SharedPtr<c_void>) -> *mut c_void;

/// The textures of the mods (`ui.textures`): `TextureBook` keeps the ids and what is due, `TexturePump` makes the game
/// calls at the next UI draw point.
///
/// A table call only queues. The pump binds a file texture through the render context (`getTexture`), uploads
/// pixels into the client texture group and binds them there, and gives back every reference the book let go.
/// A `ResourceLocation` is built once per path and kept, because the game may keep a reference to it.
pub struct TextureAdapter {
    state: &'static LoaderState,
    book: TextureBook,
    calls: TextureCalls,
    // The `ResourceLocation` of each (file system, path), by address, leaked on purpose.
    locations: Mutex<HashMap<(u8, String), usize>>,
    pump_thread: ThreadNote,
    upload_logged: AtomicBool,
    ready_word_logged: AtomicBool,
}

impl TextureAdapter {
    pub const TEXTURES: &'static str = "ui.textures";
    const AREA_VERSION: u32 = 1;
    // `ResourceFileSystem`: 0 the pack stack (the game's textures), 2 raw (an absolute path).
    const FS_USER_PACKAGE: u8 = 0;
    const FS_RAW: u8 = 2;
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    // The offsets of `ResourceLocation`: the file system, the path hash and the full hash.
    const LOCATION_FILE_SYSTEM: usize = 0;
    const LOCATION_PATH_HASH: usize = 0x28;
    const LOCATION_FULL_HASH: usize = 0x30;

    /// Offers `ui.textures` when its catalog shape resolved and the layouts fit. Logs one line. Works once per
    /// process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(|row| row.id == Self::TEXTURES).collect();
        let calls = rows.iter().any(|row| row.shape.is_some()).then(|| TextureCalls::new(state)).flatten();
        for row in &mut rows {
            if row.shape.is_some() && calls.is_none() {
                row.shape = None;
                row.missing = vec!["a texture layout that fits"];
            }
            match (row.shape, row.kind) {
                (Some(shape), NeedKind::Area) => curia::info!("area adapter", { area: row.id, shape: shape }),
                _ => curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") }),
            }
        }
        if let Some(calls) = calls {
            Self::install_with(state, calls);
        }
        rows
    }

    /// Installs the adapter over `calls` and offers `ui.textures`. False when an adapter is already installed.
    pub fn install_with(state: &'static LoaderState, calls: TextureCalls) -> bool {
        if STARTED.swap(true, Ordering::AcqRel) {
            curia::warn!("the texture adapter is already installed");
            return false;
        }
        let adapter = Self {
            state,
            book: TextureBook::new(),
            calls,
            locations: Mutex::new(HashMap::new()),
            pump_thread: ThreadNote::new(),
            upload_logged: AtomicBool::new(false),
            ready_word_logged: AtomicBool::new(false),
        };
        if ADAPTER.set(adapter).is_err() {
            return false;
        }
        state.areas().offer(Self::TEXTURES, Self::AREA_VERSION);
        true
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static TextureAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&TextureAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// The texture book.
    pub fn book(&self) -> &TextureBook {
        &self.book
    }

    fn status(refusal: TextureRefusal) -> AmmlStatus {
        match refusal {
            TextureRefusal::Stale => AmmlStatus::STALE,
            TextureRefusal::NotOwner | TextureRefusal::OutsideModFolder => AmmlStatus::REFUSED,
            TextureRefusal::EmptyPath | TextureRefusal::BadImage => AmmlStatus::INVALID_ARGUMENT,
        }
    }

    fn id_of(key: TextureKey) -> AmmlTextureId {
        AmmlTextureId { id: key.id, generation: key.generation, _reserved: 0 }
    }

    /// The key of the id `texture`.
    pub fn key_of(texture: AmmlTextureId) -> TextureKey {
        TextureKey { id: texture.id, generation: texture.generation }
    }

    fn check_area(&self) -> Result<(), AmmlStatus> {
        if self.state.areas().is_available(Self::TEXTURES, Self::AREA_VERSION) {
            Ok(())
        } else {
            Err(AmmlStatus::NOT_AVAILABLE)
        }
    }

    /// Queues the texture `path` of `source` for the mod `owner` whose folder is `mod_dir`.
    pub(crate) fn load(
        &self,
        owner: usize,
        source: AmmlTextureSource,
        path: &str,
        mod_dir: &str,
    ) -> Result<AmmlTextureId, AmmlStatus> {
        self.check_area()?;
        let kind = match source {
            AmmlTextureSource::MOD_FILE => TextureSourceKind::ModFile,
            AmmlTextureSource::VANILLA => TextureSourceKind::Vanilla,
            _ => return Err(AmmlStatus::INVALID_ARGUMENT),
        };
        let key = self.book.load(owner, kind, path, mod_dir).map_err(Self::status)?;
        TexturePump::request();
        Ok(Self::id_of(key))
    }

    /// Queues an upload of `width` x `height` RGBA pixels for the mod `owner`.
    pub(crate) fn upload(
        &self,
        owner: usize,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<AmmlTextureId, AmmlStatus> {
        self.check_area()?;
        let key = self.book.upload(owner, width, height, pixels).map_err(Self::status)?;
        TexturePump::request();
        Ok(Self::id_of(key))
    }

    /// Whether the game gave the data of `texture`.
    pub(crate) fn ready(&self, texture: AmmlTextureId) -> bool {
        self.book.ready(Self::key_of(texture))
    }

    /// Queues the free of `texture` of the mod `owner`.
    pub(crate) fn unload(&self, owner: usize, texture: AmmlTextureId) -> Result<(), AmmlStatus> {
        self.check_area()?;
        self.book.unload(owner, Self::key_of(texture)).map_err(Self::status)?;
        TexturePump::request();
        Ok(())
    }

    /// The `TexturePtr` address of `key` to draw with; `Ok(None)` while the texture is not ready, `STALE` for a
    /// texture of an older generation or one freed.
    pub(crate) fn drawable(&self, key: TextureKey) -> Result<Option<usize>, AmmlStatus> {
        self.book.drawable(key).map_err(Self::status)
    }

    /// A world was left: every texture is freed and the old ids are stale.
    pub fn world_left(&self) {
        let freed = self.book.world_left();
        TexturePump::request();
        curia::info!("textures freed", { reason: "world_left", count: freed });
    }

    /// The render device was reset: every texture is freed and the old ids are stale.
    pub fn device_reset(&self) {
        let freed = self.book.world_left();
        TexturePump::request();
        curia::info!("textures freed", { reason: "device_reset", count: freed });
        TextureReloadAdapter::with((), TextureReloadAdapter::device_reset);
    }

    /// The game may have dropped its textures (`reason` names the cause): every texture is bound again.
    pub fn rebind(&self, reason: &'static str) {
        if self.book.is_empty() {
            return;
        }
        self.book.rebind_all();
        TexturePump::request();
        curia::info!("textures rebind", { reason: reason, count: self.book.len() });
    }

    /// The mod at `owner` is disabled: its textures are freed.
    pub fn disable_owner(&self, owner: usize) {
        let freed = self.book.disable_owner(owner);
        if freed > 0 {
            TexturePump::request();
            curia::info!("textures freed", { reason: "mod_disabled", count: freed });
        }
    }

    fn elapsed_ms(&self) -> u64 {
        TexturePump::now_ms()
    }

    /// The game calls of one UI draw point with the render context `context`: gives back what the book let go,
    /// runs the jobs that are due, reads the ready word of the textures that wait, and tells the pump whether more
    /// work is left.
    pub fn pump(&self, context: usize) {
        self.pump_with(context, true);
    }

    /// Like `pump`. With `releases` false it gives nothing back: the jobs run and the releases wait for a pump that
    /// runs them.
    pub fn pump_with(&self, context: usize, releases: bool) {
        let state = self.state;
        let game = state.events().gate().on_game_thread();
        if self.pump_thread.changed(if game { AmmlThread::GAME } else { AmmlThread::RENDER }) {
            curia::info!("textures pump thread", { game: game });
        }
        let work = self.book.take_work_with(self.elapsed_ms(), releases);
        let mut group = None;
        for release in &work.releases {
            self.release(release, &mut group);
        }
        for job in work.jobs {
            match job {
                TextureJob::Bind { key, target } => self.bind(context, key, &target, &mut group),
                TextureJob::Upload { key, name, width, height, pixels } => {
                    self.upload_job(key, &name, width, height, &pixels, &mut group);
                }
            }
        }
        self.read_ready_words();
        if let Some(mut group) = group {
            // SAFETY: the pointer holds the use `getTextureGroup` took, and is released once.
            unsafe { group.release() };
        }
        TexturePump::settle(|| {
            (
                self.book.has_due_work()
                    || ItemDrawAdapter::with(false, ItemDrawAdapter::has_frees)
                    || InlineIconAdapter::with(false, InlineIconAdapter::needs_sync),
                self.book.has_waiting(),
            )
        });
    }

    // The client texture group, asked once per pump; `None` without a primary client or when the game gives none.
    fn group(&self, group: &mut Option<SharedPtr<c_void>>) -> Option<usize> {
        if group.is_none() {
            let client = SessionAdapter::primary_client()?;
            let function = ClientAdapter::virtual_fn(client, Some(self.calls.get_texture_group_slot))?;
            // SAFETY: an all-zero `shared_ptr` is the empty pointer.
            let mut out: SharedPtr<c_void> = unsafe { std::mem::zeroed() };
            // SAFETY: the slot holds `shared_ptr<TextureGroup> ClientInstance::getTextureGroup()`: this in rcx, the
            // result through the hidden pointer in rdx.
            let function = unsafe { std::mem::transmute::<usize, GetTextureGroupFn>(function) };
            // SAFETY: called on its own client with a writable result.
            unsafe { function(client, &mut out) };
            *group = Some(out);
        }
        group.as_ref()?.get().map(|group| std::ptr::from_ref(group).addr())
    }

    // Gives back one release: the name in the texture group, then the held `TexturePtr`.
    fn release(&self, release: &TextureRelease, group: &mut Option<SharedPtr<c_void>>) {
        if let Some(name) = &release.unload_name
            && let Some(group_address) = self.group(group)
            && let Some(location) = self.location(Self::FS_USER_PACKAGE, name)
        {
            // SAFETY: the entry is `TextureGroup::unloadTexture`: the group itself in rcx, the location in rdx.
            let function = unsafe { std::mem::transmute::<usize, UnloadFn>(self.calls.unload) };
            // SAFETY: the group and the location are live.
            unsafe { function(group_address, std::ptr::with_exposed_provenance(location), 0) };
        }
        Self::free_held(release.held);
    }

    // Releases the `TexturePtr` at `held` and frees its box.
    fn free_held(held: usize) {
        if held == 0 {
            return;
        }
        // SAFETY: `held` is the address of a box made by `held_box`, given back once by the book.
        let mut texture = unsafe { Box::from_raw(std::ptr::with_exposed_provenance_mut::<TexturePtrLayout>(held)) };
        // SAFETY: the box holds the two uses the game gave, released once.
        unsafe { texture.release() };
    }

    // Keeps the `TexturePtr` the game filled as `key`'s; an empty one is dropped and the book is told 0.
    fn held_box(&self, key: TextureKey, texture: Box<TexturePtrLayout>) {
        if texture.data() == 0 {
            self.book.bound(key, 0);
            return;
        }
        let held = Box::into_raw(texture).expose_provenance();
        if !self.book.bound(key, held) {
            Self::free_held(held);
        }
    }

    fn bind(&self, context: usize, key: TextureKey, target: &TextureTarget, group: &mut Option<SharedPtr<c_void>>) {
        let mut texture = Box::new(TexturePtrLayout::empty());
        match target {
            TextureTarget::Vanilla(path) | TextureTarget::Raw(path) => {
                let file_system =
                    if matches!(target, TextureTarget::Raw(_)) { Self::FS_RAW } else { Self::FS_USER_PACKAGE };
                let Some(location) = self.location(file_system, path) else { return };
                // SAFETY: `context` is the live render context of a draw call running now; its vtable holds
                // `getTexture` at the slot of the entry.
                let Some(function) = (unsafe { crate::DrawCalls::slot(context, self.calls.context_get_texture_slot) })
                else {
                    return;
                };
                // SAFETY: the slot holds `getTexture`: this in rcx, the hidden result in rdx, the location in r8 and
                // forceReload in r9b.
                let function = unsafe { std::mem::transmute::<usize, ContextGetTextureFn>(function) };
                // SAFETY: the context is live, the result is 20h writable bytes, the location is live.
                unsafe { function(context, &mut *texture, std::ptr::with_exposed_provenance(location), 0) };
            }
            TextureTarget::Group(name) => {
                let Some(group_address) = self.group(group) else { return };
                let Some(location) = self.location(Self::FS_USER_PACKAGE, name) else { return };
                // SAFETY: the entry is `TextureGroupBase::getTexture`, called on the group plus the base offset.
                let function = unsafe { std::mem::transmute::<usize, GroupGetTextureFn>(self.calls.group_get) };
                // SAFETY: the group base is live, the result is 20h writable bytes, the location is live; the
                // optional and the layer type are zero (no value, the default layer).
                unsafe {
                    function(
                        group_address + self.calls.group_base,
                        &mut *texture,
                        std::ptr::with_exposed_provenance(location),
                        0,
                        0,
                        0,
                    );
                }
            }
        }
        self.held_box(key, texture);
    }

    fn upload_job(
        &self,
        key: TextureKey,
        name: &str,
        width: u32,
        height: u32,
        pixels: &[u8],
        group: &mut Option<SharedPtr<c_void>>,
    ) {
        let Some(group_address) = self.group(group) else { return };
        let Some(location) = self.location(Self::FS_USER_PACKAGE, name) else { return };
        let mut buffer = [0u64; TextureCalls::BUFFER_CAPACITY_WORDS];
        let description = ImageDescriptionLayout::rgba(width, height);
        // SAFETY: the entry is `ImageBuffer(ImageDescription const&)`; `buffer` holds the `image_buffer_words` words
        // it builds into (`TextureCalls::new` keeps them within the capacity).
        unsafe {
            let ctor = std::mem::transmute::<usize, ImageBufferCtorFn>(self.calls.image_buffer_ctor);
            ctor(buffer.as_mut_ptr(), &description);
        }
        let word = |offset: usize| offset / size_of::<u64>();
        let data = std::ptr::with_exposed_provenance_mut::<u8>(buffer[word(self.calls.buffer_data)] as usize);
        if data.is_null() {
            return;
        }
        if usize::try_from(buffer[word(self.calls.buffer_size)]) != Ok(pixels.len()) {
            let deleter = buffer[word(self.calls.buffer_deleter)] as usize;
            if deleter != 0 {
                // SAFETY: a non-zero deleter is the blob deleter the constructor stored for `data`.
                unsafe { std::mem::transmute::<usize, BlobDeleterFn>(deleter)(data) };
            }
            return;
        }
        // SAFETY: the size check shows `data` holds exactly the image bytes.
        unsafe { std::ptr::copy_nonoverlapping(pixels.as_ptr(), data, pixels.len()) };
        // SAFETY: the entry is `TextureGroup::uploadTexture`: the group itself in rcx, the location in rdx, the buffer
        // by address in r8. The callee destroys the buffer, so it is not destroyed here.
        unsafe {
            let upload = std::mem::transmute::<usize, UploadFn>(self.calls.upload);
            upload(group_address, std::ptr::with_exposed_provenance(location), buffer.as_mut_ptr());
        }
        self.book.uploaded(key);
        if !self.upload_logged.swap(true, Ordering::Relaxed) {
            curia::info!("textures upload done", { width: width, height: height });
        }
        self.bind(0, key, &TextureTarget::Group(name.to_string()), group);
    }

    // The `ResourceLocation` of `path` in `file_system`, built once and kept; its address, or `None` when the build
    // refused (a path longer than the view allows is not possible here).
    fn location(&self, file_system: u8, path: &str) -> Option<usize> {
        let mut locations = self.locations.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = (file_system, path.to_string());
        if let Some(address) = locations.get(&entry) {
            return Some(*address);
        }
        let words = Box::new([0u64; TextureCalls::LOCATION_SIZE / size_of::<u64>()]);
        let address = Box::leak(words).as_mut_ptr().expose_provenance();
        let view = PathViewLayout { data: path.as_ptr(), size: path.len() };
        // SAFETY: the entry is `ResourceLocation(Core::PathView)`: it builds into the zeroed 38h bytes (the entry
        // `ResourceLocation.size` was checked) and copies the path.
        unsafe {
            let ctor = std::mem::transmute::<usize, LocationCtorFn>(self.calls.resource_location_ctor);
            ctor(std::ptr::with_exposed_provenance_mut(address), &view);
        }
        if file_system != Self::FS_USER_PACKAGE {
            // SAFETY: the constructor filled the 38h bytes at `address`: the hash of the path at +28h.
            let path_hash = unsafe { ObjectRead::value::<u64>(address, Self::LOCATION_PATH_HASH) }.unwrap_or(0);
            let full = (u64::from(file_system) ^ Self::FNV_OFFSET).wrapping_mul(Self::FNV_PRIME) ^ path_hash;
            // SAFETY: the object is 38h bytes and leaked, so the file system word and the full hash are writable.
            unsafe {
                std::ptr::with_exposed_provenance_mut::<u32>(address + Self::LOCATION_FILE_SYSTEM)
                    .write_unaligned(u32::from(file_system));
                std::ptr::with_exposed_provenance_mut::<u64>(address + Self::LOCATION_FULL_HASH).write_unaligned(full);
            }
        }
        locations.insert(entry, address);
        Some(address)
    }

    // Reads the ready word of every texture that waits and tells the book when it is not zero. The first value read
    // is logged: the word is not proven from the executable.
    fn read_ready_words(&self) {
        for (key, held) in self.book.waiting() {
            // SAFETY: `held` is the address of a `TexturePtr` box the book holds; its first word is the data pointer.
            let Some(data) = (unsafe { ObjectRead::pointer(held, 0) }) else { continue };
            // SAFETY: the game keeps the `BedrockTextureData` alive while the `TexturePtr` is held; its size word is
            // at the offset of `READY_WORD`.
            let Some(word) = (unsafe { ObjectRead::value::<u64>(data, TextureCalls::READY_WORD) }) else { continue };
            if !self.ready_word_logged.swap(true, Ordering::Relaxed) {
                curia::info!("textures ready word", { offset: TextureCalls::READY_WORD, value: word });
            }
            if word != 0 {
                self.book.mark_ready(key);
            }
        }
    }
}
