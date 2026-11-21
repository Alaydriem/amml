use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlContainerEntry, AmmlItemDraw, AmmlItemDrawFlags, AmmlPoint, AmmlStatus, AmmlStr};
use amml_runtime::{ItemDrawBook, ItemDrawKey, ItemDrawRefusal};

use crate::adapters::OpenUndo;
use crate::{
    ClientAdapter, ContentsEntry, ContentsList, InventoryAdapter, InventoryLayout, ItemDrawCalls, ItemDrawFrame,
    LoaderState, ObjectRead, SessionAdapter, TagReader, TexturePump, UiDrawAdapter, UiScope,
};

static ADAPTER: OnceLock<ItemDrawAdapter> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);
static THREAD_REFUSAL_LOGGED: AtomicBool = AtomicBool::new(false);
static FIRST_DRAW_LOGGED: AtomicBool = AtomicBool::new(false);
static CAP_LOGGED: AtomicBool = AtomicBool::new(false);

/// `static ItemStack ItemStack::fromTag(CompoundTag const&)`: rcx the hidden result, rdx the tag, the result returned.
type FromTagFn = unsafe extern "C" fn(*mut c_void, *const c_void) -> *mut c_void;
/// `ItemStackBase::~ItemStackBase()`: rcx the stack. It destroys the members and does not free the stack.
type StackDestroyFn = unsafe extern "C" fn(*mut c_void);
/// `BaseActorRenderContext::BaseActorRenderContext(ScreenContext&, IClientInstance&, IMinecraftGame&)`.
type ContextCtorFn = unsafe extern "C" fn(*mut c_void, usize, usize, usize) -> *mut c_void;
/// `int Item::getAnimationFrameFor(Mob*, bool, ItemStack const*, bool) const`.
type AnimationFrameFn = unsafe extern "C" fn(usize, usize, u8, usize, u8) -> i32;
/// `void ItemRenderer::renderGuiItemNew(BaseActorRenderContext&, ItemStack const&, int frame, float x, float y,
/// bool foil, float transparency, float light, float scale, int zOrder)`.
type RenderGuiItemFn = unsafe extern "C" fn(usize, usize, usize, i32, f32, f32, u8, f32, f32, f32, i32);

// The words of the buffer a stack copy lives in: at least `InventoryLayout::STACK_SIZE` bytes.
const COPY_WORDS: usize = 10;
const _: () = assert!(COPY_WORDS * size_of::<u128>() >= InventoryLayout::STACK_SIZE as usize);

/// The item stack copies of the mods (`inventory.read` version 4 and `UiDraw::item`): the contents of a shulker box or
/// a bundle, and the draws of the copies.
///
/// A copy is a game `ItemStack` that `ItemStack::fromTag` built into a loader buffer from one tag of a contents list.
/// `ItemDrawBook` names it and queues its free: a mod may drop its handle on any thread, and the game stack is
/// destroyed, with its buffer, at the next UI draw point on the game thread (`TexturePump` calls `pump`) or at a world
/// leave. A contents list is read and closed by the mod in the same call that opened it (`contents_open`,
/// `contents_entry`, `contents_close`): an entry the mod did not read is freed when the list closes.
///
/// An item draw builds one `BaseActorRenderContext` for a `ScreenRender` dispatch and keeps it until the dispatch
/// ends (`finish`).
pub struct ItemDrawAdapter {
    state: &'static LoaderState,
    book: ItemDrawBook,
    tags: Option<TagReader>,
    from_tag: usize,
    stack_dtor: usize,
    calls: Option<ItemDrawCalls>,
    contents: Mutex<HashMap<u64, ContentsList>>,
    next_contents: AtomicU64,
    frame: Mutex<Option<ItemDrawFrame>>,
}

impl ItemDrawAdapter {
    // The version of `inventory.read` that serves the contents calls.
    const CONTENTS_VERSION: u32 = 4;
    // The key of the item list of a shulker box and of the contents list of a bundle, in the user data tag.
    const ITEMS: &'static str = "Items";
    const BUNDLE_CONTENTS: &'static str = "storage_item_component_content";
    const BUNDLE_WEIGHT: &'static str = "bundle_weight";
    const SLOT: &'static str = "Slot";
    // The deepest nesting of contents a read follows: a shulker box, its bundle, and one more for a loop in a tag.
    const MAX_DEPTH: usize = 3;
    /// The most copies one contents open makes: the 27 slots of a box and 64 items for each of them as a bundle. A
    /// bundle past it keeps no contents; `ItemDrawBook::MAX_PER_OWNER` bounds the copies of a mod over every open.
    pub const MAX_PER_OPEN: usize = 27 + 27 * 64;

    /// Builds the adapter from the resolved entries: the contents calls need `ItemStack.fromTag` and the tag map
    /// anchor, the draw needs `ItemDrawCalls`. Logs one line. Works once per process.
    pub fn install(state: &'static LoaderState) {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let tags = value("CompoundTag.nodeVariantIndex")
            .zip(value("CompoundTag.vtable"))
            .and_then(|(index, vtable)| TagReader::new(index as u64, vtable));
        let from_tag = value("ItemStack.fromTag").unwrap_or(0);
        let stack_dtor = value("ItemStackBase.dtor").unwrap_or(0);
        Self::install_with(state, tags, from_tag, stack_dtor, ItemDrawCalls::new(state));
    }

    /// Installs the adapter over the given calls. False when an adapter is already installed.
    pub fn install_with(
        state: &'static LoaderState,
        tags: Option<TagReader>,
        from_tag: usize,
        stack_dtor: usize,
        calls: Option<ItemDrawCalls>,
    ) -> bool {
        if STARTED.swap(true, Ordering::AcqRel) {
            curia::warn!("the item draw adapter is already installed");
            return false;
        }
        let adapter = Self {
            state,
            book: ItemDrawBook::new(),
            tags,
            from_tag,
            stack_dtor,
            calls,
            contents: Mutex::new(HashMap::new()),
            next_contents: AtomicU64::new(1),
            frame: Mutex::new(None),
        };
        curia::info!("item draw adapter", { contents: adapter.contents_ready(), draw: adapter.calls.is_some() });
        ADAPTER.set(adapter).is_ok()
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static ItemDrawAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&ItemDrawAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// The book of the copies.
    pub fn book(&self) -> &ItemDrawBook {
        &self.book
    }

    /// The reader of the game's `CompoundTag`s; `None` when the tag layout entries did not resolve or do not fit.
    pub(crate) fn tag_reader(&self) -> Option<TagReader> {
        self.tags
    }

    fn contents_ready(&self) -> bool {
        self.tags.is_some() && self.from_tag != 0 && self.stack_dtor != 0
    }

    fn lists(&self) -> MutexGuard<'_, HashMap<u64, ContentsList>> {
        self.contents.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn frame_lock(&self) -> MutexGuard<'_, Option<ItemDrawFrame>> {
        self.frame.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn key_of(draw: AmmlItemDraw) -> ItemDrawKey {
        ItemDrawKey { id: draw.id, generation: draw.generation }
    }

    fn status(refusal: ItemDrawRefusal) -> AmmlStatus {
        match refusal {
            ItemDrawRefusal::Stale => AmmlStatus::STALE,
            ItemDrawRefusal::NotOwner | ItemDrawRefusal::TooMany => AmmlStatus::REFUSED,
        }
    }

    /// The index of the live mod `module`: `INVALID_ARGUMENT` for an unknown one, `REFUSED` for one that is not live.
    ///
    /// # Safety
    /// `module` is valid for the call.
    pub(crate) unsafe fn live_owner(module: AmmlStr) -> Result<usize, AmmlStatus> {
        let state = LoaderState::get().ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the caller promises `module` is valid for the call.
        let module = unsafe { module.as_str() }?;
        state.live_mod(module)
    }

    /// Opens the contents of `stack` for the mod `owner`: the stack must be one `player` holds or one lent in the
    /// running UI scope (`REFUSED` otherwise, and off the game thread). Gives the contents id and the number of
    /// entries; both are 0 for a stack with no tag list. `REFUSED` when the top level alone would pass the live draws of the mod; a bundle that does not fit keeps no contents.
    pub(crate) fn open(
        &self,
        owner: usize,
        player: *const c_void,
        stack: *const c_void,
    ) -> Result<(u64, u32), AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(InventoryAdapter::READ, Self::CONTENTS_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !gate.on_game_thread() {
            if !THREAD_REFUSAL_LOGGED.swap(true, Ordering::AcqRel) {
                curia::info!("item contents refused", { thread: "not game" });
            }
            return Err(AmmlStatus::REFUSED);
        }
        let reader = self.tags.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        if !self.contents_ready() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let inventory = InventoryAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let (stack, layout) = inventory.checked_stack(player, stack)?;
        let user_data = InventoryAdapter::user_data_of(stack, layout);
        if user_data == 0 {
            return Ok((0, 0));
        }
        // SAFETY: `user_data` is the live CompoundTag of a stack the loader checked.
        let elements =
            unsafe { reader.list(user_data, Self::ITEMS).or_else(|| reader.list(user_data, Self::BUNDLE_CONTENTS)) };
        let Some(elements) = elements else { return Ok((0, 0)) };
        let mut undo = OpenUndo::default();
        match self.build(owner, &elements, 0, inventory, layout, reader, &mut undo) {
            Ok(entries) => {
                let len = u32::try_from(entries.len()).unwrap_or(u32::MAX);
                Ok((self.insert(owner, entries), len))
            }
            Err(status) => {
                self.undo(owner, undo);
                Err(status)
            }
        }
    }

    fn insert(&self, owner: usize, entries: Vec<ContentsEntry>) -> u64 {
        let id = self.next_contents.fetch_add(1, Ordering::Relaxed);
        self.lists().insert(id, ContentsList { owner, entries });
        id
    }

    // The entries of the list `elements`: a copy of each stack the game can make, with the fields the mod reads.
    #[allow(clippy::too_many_arguments)]
    fn build(
        &self,
        owner: usize,
        elements: &[usize],
        depth: usize,
        inventory: &InventoryAdapter,
        layout: &InventoryLayout,
        reader: TagReader,
        undo: &mut OpenUndo,
    ) -> Result<Vec<ContentsEntry>, AmmlStatus> {
        let mut entries = Vec::new();
        // The bundles whose contents are built after every entry of this list has its copy, so a full cap leaves the
        // entries of the list and costs only the contents of the last bundles.
        let mut pending: Vec<(usize, Vec<usize>)> = Vec::new();
        for (index, element) in elements.iter().enumerate() {
            // SAFETY: each element is a live Tag of a list the loader read.
            let Some(compound) = (unsafe { reader.compound(*element) }) else { continue };
            let Some(address) = self.make_copy(compound, layout) else { continue };
            let added = if undo.keys.len() < Self::MAX_PER_OPEN { self.book.add(owner, address).ok() } else { None };
            let Some(key) = added else {
                self.destroy_copy(address);
                return Err(AmmlStatus::REFUSED);
            };
            undo.keys.push(key);
            let mut entry = AmmlContainerEntry::empty();
            // SAFETY: `compound` is a live CompoundTag.
            let slot = unsafe { reader.byte(compound, Self::SLOT) };
            entry.slot =
                slot.and_then(|slot| u8::try_from(slot).ok()).unwrap_or_else(|| u8::try_from(index).unwrap_or(u8::MAX));
            entry.item = inventory.fields_of(address, layout);
            entry.draw = AmmlItemDraw { id: key.id, generation: key.generation, _reserved: 0 };
            let name = InventoryAdapter::name_of(address, layout).unwrap_or_default();
            let user_data = InventoryAdapter::user_data_of(address, layout);
            if user_data != 0 && Self::is_bundle(&name) {
                // SAFETY: `user_data` is the live CompoundTag of the copy.
                if let Some(weight) = unsafe { reader.int(user_data, Self::BUNDLE_WEIGHT) } {
                    entry.has_bundle_weight = 1;
                    entry.bundle_weight = weight;
                }
                // SAFETY: as above.
                let nested = unsafe { reader.list(user_data, Self::BUNDLE_CONTENTS) };
                if let Some(nested) = nested.filter(|_| depth < Self::MAX_DEPTH) {
                    pending.push((entries.len(), nested));
                }
            }
            entries.push(ContentsEntry { entry, name, taken: false });
        }
        for (index, nested) in pending {
            let mark = (undo.keys.len(), undo.lists.len());
            match self.build(owner, &nested, depth + 1, inventory, layout, reader, undo) {
                Ok(list) => {
                    let id = self.insert(owner, list);
                    undo.lists.push(id);
                    entries[index].entry.bundle = id;
                }
                Err(_) => {
                    // The mod holds the most draws it may: this bundle keeps no contents.
                    self.undo_to(owner, undo, mark);
                    if !CAP_LOGGED.swap(true, Ordering::Relaxed) {
                        curia::info!("bundle contents left out", { reason: "draw cap" });
                    }
                }
            }
        }
        Ok(entries)
    }

    // Whether a type name is a bundle: it ends in `bundle`, after the `minecraft:` prefix.
    fn is_bundle(name: &str) -> bool {
        name.strip_prefix("minecraft:").unwrap_or(name).ends_with("bundle")
    }

    // Gives back what a failed open made.
    fn undo(&self, owner: usize, mut undo: OpenUndo) {
        self.undo_to(owner, &mut undo, (0, 0));
    }

    // Gives back what was made after `mark` (the lengths of the key and list records).
    fn undo_to(&self, owner: usize, undo: &mut OpenUndo, mark: (usize, usize)) {
        let lists = undo.lists.split_off(mark.1);
        let keys = undo.keys.split_off(mark.0);
        let mut map = self.lists();
        for id in &lists {
            map.remove(id);
        }
        drop(map);
        for key in keys {
            let _ = self.book.release(owner, key);
        }
        TexturePump::request();
    }

    // A game stack made from `tag` in a loader buffer; its address, or `None` for a tag of an item the game does not
    // know (the stack is then destroyed).
    fn make_copy(&self, tag: usize, layout: &InventoryLayout) -> Option<usize> {
        let address = Box::into_raw(Box::new([0u128; COPY_WORDS])).expose_provenance();
        // SAFETY: the entry is the address of `static ItemStack ItemStack::fromTag(CompoundTag const&)`.
        let from_tag = unsafe { std::mem::transmute::<usize, FromTagFn>(self.from_tag) };
        // SAFETY: the buffer is at least `ItemStack.size` writable bytes (the layout was checked against the mirror)
        // and `tag` is a live CompoundTag.
        unsafe { from_tag(std::ptr::with_exposed_provenance_mut(address), std::ptr::with_exposed_provenance(tag)) };
        if InventoryAdapter::item_of(address, layout) == 0 {
            self.destroy_copy(address);
            return None;
        }
        Some(address)
    }

    // Destroys the game stack at `address` through the resolved `ItemStackBase.dtor` and gives back its buffer. A copy
    // made by `fromTag` holds no request id, so the body that `ItemStack::~ItemStack` jumps to destroys all of it.
    fn destroy_copy(&self, address: usize) {
        if self.stack_dtor != 0 {
            // SAFETY: the entry is the address of `ItemStackBase::~ItemStackBase()`: the stack in rcx.
            let destroy = unsafe { std::mem::transmute::<usize, StackDestroyFn>(self.stack_dtor) };
            // SAFETY: the stack is live and destroyed once.
            unsafe { destroy(std::ptr::with_exposed_provenance_mut(address)) };
        }
        // SAFETY: the address is a box of `COPY_WORDS` words made by `make_copy` and freed once.
        drop(unsafe { Box::from_raw(std::ptr::with_exposed_provenance_mut::<[u128; COPY_WORDS]>(address)) });
    }

    /// Writes entry `index` of the list `contents` in `out` (as far as the caller's `size` covers) and gives the
    /// entry's draw and bundle list to the caller. `STALE` for a list that is closed.
    pub(crate) fn entry(&self, owner: usize, contents: u64, index: u32) -> Result<AmmlContainerEntry, AmmlStatus> {
        let mut lists = self.lists();
        let list = lists.get_mut(&contents).ok_or(AmmlStatus::STALE)?;
        if list.owner != owner {
            return Err(AmmlStatus::REFUSED);
        }
        let entry = list.entries.get_mut(index as usize).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        if entry.taken {
            return Err(AmmlStatus::REFUSED);
        }
        entry.taken = true;
        Ok(entry.entry)
    }

    /// The type name of entry `index` of the list `contents`.
    pub(crate) fn entry_name(&self, owner: usize, contents: u64, index: u32) -> Result<String, AmmlStatus> {
        let lists = self.lists();
        let list = lists.get(&contents).ok_or(AmmlStatus::STALE)?;
        if list.owner != owner {
            return Err(AmmlStatus::REFUSED);
        }
        list.entries.get(index as usize).map(|entry| entry.name.clone()).ok_or(AmmlStatus::INVALID_ARGUMENT)
    }

    /// Closes the list `contents`: the draws of the entries nobody took are freed, and so are their bundle lists.
    /// A list that is closed already does nothing.
    pub(crate) fn close(&self, owner: usize, contents: u64) {
        if self.lists().get(&contents).is_some_and(|list| list.owner != owner) {
            return;
        }
        self.close_list(contents);
    }

    // Closes the list `contents`, whoever owns it.
    fn close_list(&self, contents: u64) {
        let Some(list) = self.lists().remove(&contents) else { return };
        let mut nested = Vec::new();
        for entry in list.entries.iter().filter(|entry| !entry.taken) {
            let draw = entry.entry.draw;
            let _ = self.book.release(list.owner, Self::key_of(draw));
            if entry.entry.bundle != 0 {
                nested.push(entry.entry.bundle);
            }
        }
        for id in nested {
            self.close_list(id);
        }
        TexturePump::request();
    }

    /// Queues the free of the draw `draw` of the mod `owner`. Allowed on any thread.
    pub(crate) fn release(&self, owner: usize, draw: AmmlItemDraw) -> Result<(), AmmlStatus> {
        self.book.release(owner, Self::key_of(draw)).map_err(Self::status)?;
        TexturePump::request();
        Ok(())
    }

    /// Whether a game stack waits to be destroyed.
    pub fn has_frees(&self) -> bool {
        self.book.has_frees()
    }

    /// At a UI draw point: destroys the game stacks whose handles were dropped. They stay queued off the game thread.
    pub fn pump(&self) {
        if !self.book.has_frees() || !self.state.events().gate().on_game_thread() {
            return;
        }
        for address in self.book.take_frees() {
            self.destroy_copy(address);
        }
    }

    /// A world was left: every copy is freed, every contents list is closed and the old ids are stale.
    pub fn world_left(&self) {
        let freed = self.book.world_left();
        self.lists().clear();
        TexturePump::request();
        curia::info!("item draws freed", { reason: "world_left", count: freed });
    }

    /// The mod at `owner` is disabled: its copies are freed and its contents lists closed.
    pub fn disable_owner(&self, owner: usize) {
        let freed = self.book.disable_owner(owner);
        self.lists().retain(|_, list| list.owner != owner);
        if freed > 0 {
            TexturePump::request();
            curia::info!("item draws freed", { reason: "mod_disabled", count: freed });
        }
    }

    /// Draws the copy `draw` of the mod `owner` with its top left corner at `at` in `view`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        &self,
        owner: usize,
        view: u64,
        draw: AmmlItemDraw,
        at: AmmlPoint,
        flags: AmmlItemDrawFlags,
        alpha: f32,
        z: i32,
    ) -> Result<(), AmmlStatus> {
        if flags.has_unknown_bits() || ![at.x, at.y, alpha].iter().all(|value| value.is_finite()) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let calls = self.calls.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let context = UiDrawAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.before_item(view))?;
        let key = Self::key_of(draw);
        if self.book.owner(key).map_err(Self::status)? != owner {
            return Err(AmmlStatus::REFUSED);
        }
        let address = self.book.address(key).map_err(Self::status)?;
        let inventory = InventoryAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let layout = inventory.layout().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let mut frame = self.frame_lock();
        let (renderer, holder, object) = {
            let frame = self.frame_for(&mut frame, context, &calls)?;
            (frame.renderer, frame.holder, frame.address())
        };
        let item = InventoryAdapter::item_of(address, layout);
        let animation = Self::animation_frame(&calls, item, holder, address);
        // SAFETY: the entry is the address of `ItemRenderer::renderGuiItemNew`: this in rcx, the context in rdx, the
        // stack in r8, the frame in r9d, then on the stack x, y, foil, transparency, light, scale and the z order.
        let render = unsafe { std::mem::transmute::<usize, RenderGuiItemFn>(calls.render_gui_item) };
        // SAFETY: the renderer and the context are live for the dispatch, the stack is a live copy the book holds
        // (it is freed only at a UI draw point, never during this call).
        unsafe {
            render(
                renderer,
                object,
                address,
                animation,
                at.x,
                at.y,
                u8::from(flags.contains(AmmlItemDrawFlags::FOIL_ONLY)),
                alpha,
                1.0,
                1.0,
                z,
            );
        }
        if !FIRST_DRAW_LOGGED.swap(true, Ordering::Relaxed) {
            curia::info!("first item draw", { frame: animation, z: z });
        }
        Ok(())
    }

    // The animation frame of `item` for `holder`; 0 for an item without one.
    fn animation_frame(calls: &ItemDrawCalls, item: usize, holder: usize, stack: usize) -> i32 {
        let Some(function) = InventoryAdapter::virtual_fn(item, calls.animation_slot) else { return 0 };
        // SAFETY: the slot holds `int Item::getAnimationFrameFor(Mob*, bool, ItemStack const*, bool) const`: this in
        // rcx, the holder in rdx, the first bool in r8b, the stack in r9, the second bool on the stack.
        let function = unsafe { std::mem::transmute::<usize, AnimationFrameFn>(function) };
        // SAFETY: the item and the stack are live; the holder is null or the live local player.
        unsafe { function(item, holder, 0, stack, 1) }
    }

    // The frame of the running dispatch, built when it is not there yet or belongs to an older dispatch.
    fn frame_for<'a>(
        &self,
        slot: &'a mut Option<ItemDrawFrame>,
        context: usize,
        calls: &ItemDrawCalls,
    ) -> Result<&'a ItemDrawFrame, AmmlStatus> {
        let serial = UiScope::serial();
        if slot.as_ref().is_none_or(|frame| frame.serial != serial) {
            *slot = None;
            *slot = Some(self.build_frame(serial, context, calls)?);
        }
        slot.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    fn build_frame(&self, serial: u64, context: usize, calls: &ItemDrawCalls) -> Result<ItemDrawFrame, AmmlStatus> {
        // SAFETY: `before_item` proved `context` is the live render context of a running scope; it holds its
        // `ScreenContext*` at the entry's offset.
        let screen = unsafe { ObjectRead::pointer(context, calls.screen_context) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let client = SessionAdapter::primary_client().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let game = ClientAdapter::call_pointer(client, Some(calls.get_game_slot))?;
        if game == 0 {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let mut buffer = vec![0u128; calls.context_size.div_ceil(size_of::<u128>())];
        // SAFETY: the entry is the address of the constructor: this in rcx, the references in rdx, r8 and r9.
        let ctor = unsafe { std::mem::transmute::<usize, ContextCtorFn>(calls.context_ctor) };
        // SAFETY: the buffer is `BaseActorRenderContext.size` bytes and 16-byte aligned; the three objects are live.
        unsafe { ctor(buffer.as_mut_ptr().cast(), screen, client, game) };
        let object = buffer.as_ptr().addr();
        // SAFETY: the constructed context holds its `ItemRenderer*` at the offset.
        let renderer = unsafe { ObjectRead::pointer(object, ItemDrawCalls::ITEM_RENDERER_OFFSET) };
        let holder = ClientAdapter::get().and_then(ClientAdapter::primary_local_player).unwrap_or(0);
        let frame = ItemDrawFrame::new(serial, renderer.unwrap_or(0), holder, buffer, calls.context_dtor);
        // A context with no item renderer cannot draw; the frame still drops and destroys the object.
        if renderer.is_none() { Err(AmmlStatus::NOT_AVAILABLE) } else { Ok(frame) }
    }

    /// After the last `ScreenRender` listener: destroys the render context of the dispatch.
    pub fn finish(&self) {
        *self.frame_lock() = None;
    }
}
