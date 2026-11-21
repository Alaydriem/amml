use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlBlockPos, AmmlEvent, AmmlPhase, AmmlThread, BlockChanged};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::MemberCall;
use amml_runtime::{BlockChange, BlockChangeQueue, BlockTypeBook, ImageLayout};

use crate::{
    ActorAdapter, BlockListenerAttach, BlockListenerDetours, BlockListenerObject, EventHub, LoaderState, ThreadNote,
    WorldAdapter,
};

static ADAPTER: OnceLock<BlockListener> = OnceLock::new();

/// `void BlockSource::addListener(BlockSourceListener&)` and `removeListener`: rcx the block source, rdx the listener.
type ListenerFn = unsafe extern "C" fn(usize, usize);

/// The `world.block_changed` event: a block of the dimension of the local player changed.
///
/// The loader owns a `BlockSourceListener` object (`BlockListenerObject`: a copy of the base listener vtable whose
/// `onSourceDestroyed` and `onBlockChanged` slots are loader thunks). It is attached to the block source of the local
/// player only while a mod listens to `BlockChanged`, and only on the game thread:
///
/// - at each `ClientTick` (and so at `WorldJoined`) `reconcile` attaches it when a mod listens and a block source is
///   there, and detaches it when the last listener dropped, when the world epoch changed or when the block source of
///   the local player is another one (a dimension change that has completed);
/// - `WorldLeft` detaches it before the game leaves the level; `onSourceDestroyed` forgets it without a call to the
///   game, because the source is going away. A detach asked from another thread (the last listener dropping) waits for
///   the next `ClientTick`. The listener object is made once and never freed, so a game call on another thread that
///   holds its pointer stays valid.
///
/// A change on the game thread is raised at once, after the game's own change. A change on another thread goes to the
/// `BlockChangeQueue` and is raised at the next `FrameUpdate` After. The block types of a change are read through the
/// column reads' `BlockTypeReader`, on the game thread only.
pub struct BlockListener {
    state: &'static LoaderState,
    // The absolute address of `BlockSourceListener.vtable`, and the slots the loader replaces.
    base_vtable: usize,
    destroyed_slot: usize,
    changed_slot: usize,
    // The slots of `addListener`, `removeListener` and `getDimensionId` in the `BlockSource` vtable.
    add_slot: usize,
    remove_slot: usize,
    dimension_slot: usize,
    // Whether a mod listens to `BlockChanged`; set by the event hub on every change of the listener set.
    wanted: AtomicBool,
    // Whether the thunks raise: true from the attach until a detach, a world leave or a destroyed source.
    active: AtomicBool,
    // Set by `onSourceDestroyed` for the attached source; the next `reconcile` forgets the attach.
    destroyed: AtomicBool,
    // Whether the queue may hold a change.
    queued: AtomicBool,
    // The dimension id and the epoch of the attach, for the thunks.
    dimension: AtomicI32,
    epoch: AtomicU64,
    // The one listener object of the process, made at the first attach and never freed.
    object: OnceLock<usize>,
    attached: Mutex<Option<BlockListenerAttach>>,
    queue: BlockChangeQueue,
    thread: ThreadNote,
}

impl BlockListener {
    pub const BLOCK_CHANGED: &'static str = BlockChanged::ID;

    /// Offers `world.block_changed` when its catalog shape resolved, the chunk adapter can read block types and the
    /// listener vtable is wide enough for the two slots, and logs one `event offered` or `event not available` line.
    /// Returns the row. Works once per process: a second call offers nothing and gives the catalog row. Install after
    /// `ActorAdapter` and `WorldAdapter`. `image` gives the layout of the game image, called only when every entry resolved: a build whose listener table is
    /// longer than nine slots (the word after the table is a pointer into code) is refused.
    pub fn install(state: &'static LoaderState, image: impl FnOnce() -> ImageLayout) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Event && row.id == Self::BLOCK_CHANGED)
            .collect();
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let types_ready = WorldAdapter::get().is_some_and(|world| world.block_types().is_some());
        let actor_ready = ActorAdapter::get().is_some();
        let adapter = Self::from_entries(state, value).filter(|_| types_ready && actor_ready);
        let slots_ok = adapter.as_ref().is_none_or(|adapter| adapter.table_ends_after_slots(&image()));
        let adapter = adapter.filter(|_| slots_ok);
        let offered = adapter.is_some();
        if let Some(adapter) = adapter {
            adapter.wanted.store(state.events().has_listeners(Self::BLOCK_CHANGED), Ordering::Release);
            if ADAPTER.set(adapter).is_err() {
                curia::warn!("the block listener is already installed");
                return rows;
            }
        }
        for row in &mut rows {
            if row.shape.is_some() && offered {
                state.events().offer(row.id, row.version);
            } else if row.shape.is_some() {
                row.shape = None;
                row.missing = vec![if slots_ok { "chunk adapter" } else { "BlockSourceListener slot count" }];
            }
            match row.shape {
                Some(shape) => curia::info!("event offered", { event: row.id, shape: shape }),
                None => curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    // The adapter over the resolved entries; `None` when one is missing or a replaced slot lies outside the vtable.
    fn from_entries(state: &'static LoaderState, value: impl Fn(&str) -> Option<usize>) -> Option<Self> {
        let destroyed_slot = value("BlockSourceListener.onSourceDestroyed")?;
        let changed_slot = value("BlockSourceListener.onBlockChanged")?;
        if destroyed_slot >= BlockListenerObject::SLOTS || changed_slot >= BlockListenerObject::SLOTS {
            return None;
        }
        Some(Self {
            state,
            base_vtable: value("BlockSourceListener.vtable").filter(|address| *address != 0)?,
            destroyed_slot,
            changed_slot,
            add_slot: value("BlockSource.addListener")?,
            remove_slot: value("BlockSource.removeListener")?,
            dimension_slot: value("BlockSource.getDimensionId")?,
            wanted: AtomicBool::new(false),
            active: AtomicBool::new(false),
            destroyed: AtomicBool::new(false),
            queued: AtomicBool::new(false),
            dimension: AtomicI32::new(0),
            epoch: AtomicU64::new(0),
            object: OnceLock::new(),
            attached: Mutex::new(None),
            queue: BlockChangeQueue::new(),
            thread: ThreadNote::new(),
        })
    }

    // Whether the word after the nine slots of the listener table is not a pointer into the exe's code, which proves the
    // table has no tenth slot (on 26.63 and 70.26 it is the RTTI locator of the next table). A word whose own address
    // lies outside the image is not read, and the table is refused.
    fn table_ends_after_slots(&self, image: &ImageLayout) -> bool {
        let at = self.base_vtable.wrapping_add(BlockListenerObject::SLOTS * std::mem::size_of::<usize>());
        let rva = (at as u64).wrapping_sub(self.state.module_base());
        if !image.contains(rva.saturating_add(std::mem::size_of::<usize>() as u64 - 1)) {
            return false;
        }
        // SAFETY: the table lies in the read-only data of the running game, and the word after its ninth slot is the
        // next object there, readable.
        let word = unsafe { std::ptr::with_exposed_provenance::<usize>(at).read_unaligned() };
        let rva = (word as u64).wrapping_sub(self.state.module_base());
        !image.is_executable(rva)
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&BlockListener) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// Tells the adapter whether a mod listens to `BlockChanged`. Any thread; the attach or detach follows at the next
    /// `ClientTick` on the game thread.
    pub(crate) fn set_wanted(wanted: bool) {
        if let Some(adapter) = ADAPTER.get() {
            adapter.wanted.store(wanted, Ordering::Release);
        }
    }

    /// The block source the listener is attached to; `None` while it is not attached.
    pub fn attached_source() -> Option<usize> {
        ADAPTER.get().and_then(|adapter| adapter.current().map(|attach| attach.source))
    }

    /// Attaches, moves or detaches the listener to match what the mods want and where the local player is. Game
    /// thread only; another thread does nothing. Called after `WorldJoined` and before each `ClientTick`.
    pub(crate) fn reconcile(&self) {
        let gate = self.state.events().gate();
        if !gate.on_game_thread() {
            return;
        }
        let epoch = gate.epoch().0;
        let wanted = self.wanted.load(Ordering::Acquire) && epoch != 0;
        let mut attached = self.lock_attach();
        if self.destroyed.swap(false, Ordering::AcqRel)
            && let Some(attach) = attached.take()
        {
            self.release(attach, false);
        }
        let current =
            if wanted { ActorAdapter::get().and_then(|actor| actor.primary_block_source().ok()) } else { None };
        let stale = attached.as_ref().is_some_and(|attach| current != Some(attach.source) || attach.epoch != epoch);
        if stale && let Some(attach) = attached.take() {
            self.release(attach, true);
        }
        if attached.is_none()
            && let Some(source) = current
        {
            *attached = self.attach(source, epoch);
        }
    }

    /// Before the game leaves the level: forgets the queued changes and detaches the listener while the block source is
    /// alive. From another thread the object stays until `onSourceDestroyed` or the next `ClientTick`, and no change is
    /// raised meanwhile.
    pub(crate) fn world_left(&self) {
        self.active.store(false, Ordering::Release);
        self.queue.clear();
        self.queued.store(false, Ordering::Release);
        if !self.state.events().gate().on_game_thread() {
            return;
        }
        let mut attached = self.lock_attach();
        if let Some(attach) = attached.take() {
            let remove = !self.destroyed.swap(false, Ordering::AcqRel);
            self.release(attach, remove);
        }
    }

    /// Raises the changes made off the game thread, in order, and logs the number dropped for lack of room. Called on the
    /// game thread after the `FrameUpdate` After listeners.
    pub(crate) fn flush(&self) {
        if !self.queued.swap(false, Ordering::AcqRel) {
            return;
        }
        let epoch = self.state.events().gate().epoch().0;
        let dropped = self.queue.take(epoch, |change| self.raise(change, epoch));
        if dropped > 0 {
            curia::warn!("block changes dropped", { count: dropped });
        }
    }

    /// One `onBlockChanged` call of the game for the attached block source: raised at once on the game thread, else
    /// kept for `flush`.
    pub(crate) fn changed(&self, _source: usize, pos: [i32; 3], layer: u32, new_block: usize, old_block: usize) {
        if !self.active.load(Ordering::Acquire) {
            return;
        }
        let gate = self.state.events().gate();
        let epoch = gate.epoch().0;
        if epoch == 0 || epoch != self.epoch.load(Ordering::Acquire) {
            return;
        }
        let change = BlockChange {
            dimension: self.dimension.load(Ordering::Acquire),
            pos: AmmlBlockPos { x: pos[0], y: pos[1], z: pos[2] },
            layer,
            new_block,
            old_block,
        };
        let game = gate.on_game_thread();
        if self.thread.changed(if game { AmmlThread::GAME } else { AmmlThread::RENDER }) {
            curia::info!("block changed thread", { game: game });
        }
        if game {
            self.raise(&change, epoch);
        } else {
            self.queued.store(true, Ordering::Release);
            self.queue.push(epoch, change);
        }
    }

    /// The game destroys `source`, which is the block source the listener is attached to: the listener is not raised
    /// again and the attach is forgotten at the next `reconcile`, with no call to the game.
    pub(crate) fn source_destroyed(&self, source: usize) {
        if self.current().is_some_and(|attach| attach.source == source) {
            self.active.store(false, Ordering::Release);
            self.destroyed.store(true, Ordering::Release);
        }
    }

    // Raises one change on the game thread, with the block types read now.
    fn raise(&self, change: &BlockChange, epoch: u64) {
        let types = WorldAdapter::get().and_then(WorldAdapter::block_types);
        let id = |block: usize| {
            // SAFETY: a `Block` the game passed to `onBlockChanged` (or held in the queue since) is a live block of
            // the world, and the call runs on the game thread.
            types
                .and_then(|types| unsafe { types.walk_block(block, epoch) })
                .map_or(BlockTypeBook::NONE, |walk| walk.id)
        };
        let mut event = BlockChanged {
            header: EventHub::header::<BlockChanged>(AmmlPhase::AFTER),
            dimension: change.dimension,
            pos: change.pos,
            layer: change.layer,
            new: id(change.new_block),
            old: id(change.old_block),
        };
        let hub = self.state.events();
        hub.raise(&mut event, &hub.loader_context());
    }

    // Makes the listener object, attaches it to `source` and records it; `None` with a log line when the game's calls
    // are not there.
    fn attach(&self, source: usize, epoch: u64) -> Option<BlockListenerAttach> {
        let Some(dimension) = self.read_dimension(source) else {
            curia::warn!("the block listener was not attached: no dimension id");
            return None;
        };
        let Some(add) = Self::virtual_fn(source, self.add_slot) else {
            curia::warn!("the block listener was not attached: no addListener");
            return None;
        };
        let object = match self.object.get() {
            Some(object) => *object,
            None => {
                let replaced = [
                    (self.destroyed_slot, BlockListenerDetours::destroyed_address()),
                    (self.changed_slot, BlockListenerDetours::changed_address()),
                ];
                // SAFETY: `base_vtable` is the derived address of `BlockSourceListener.vtable` of the running game;
                // `install` checked that its table has `SLOTS` slots, and the word before it is the locator.
                let made = unsafe { BlockListenerObject::new(self.base_vtable, replaced) }?.into_address();
                *self.object.get_or_init(|| made)
            }
        };
        self.dimension.store(dimension, Ordering::Release);
        self.epoch.store(epoch, Ordering::Release);
        self.destroyed.store(false, Ordering::Release);
        BlockListenerDetours::set_object(object);
        self.active.store(true, Ordering::Release);
        // SAFETY: the slot holds `void BlockSource::addListener(BlockSourceListener&)`; `source` is the live block
        // source of the local player and `object` a live listener object, called on the game thread.
        unsafe { std::mem::transmute::<usize, ListenerFn>(add)(source, object) };
        curia::info!("block listener attached", { dimension: dimension });
        Some(BlockListenerAttach { object, source, dimension, epoch })
    }

    // Ends the attach: the listener is taken out of the block source (`remove`; false when the source is gone), the
    // thunks stop answering. The object stays allocated for the next attach. Game thread only.
    fn release(&self, attach: BlockListenerAttach, remove: bool) {
        self.active.store(false, Ordering::Release);
        if remove && let Some(function) = Self::virtual_fn(attach.source, self.remove_slot) {
            // SAFETY: the slot holds `void BlockSource::removeListener(BlockSourceListener&)`; the source is alive (it
            // has not called `onSourceDestroyed`) and the object is in its listener list, called on the game thread.
            unsafe { std::mem::transmute::<usize, ListenerFn>(function)(attach.source, attach.object) };
        }
        BlockListenerDetours::set_object(0);
        curia::info!("block listener detached", { dimension: attach.dimension });
    }

    // The dimension id of the block source `source`: `BlockSource::getDimensionId`, which returns its `DimensionType`
    // (one int) through a hidden pointer.
    fn read_dimension(&self, source: usize) -> Option<i32> {
        let function = Self::virtual_fn(source, self.dimension_slot)?;
        // SAFETY: the slot holds `DimensionType BlockSource::getDimensionId() const`, a member function that returns a
        // struct of one int through a hidden pointer; `source` is the live block source, on the game thread.
        Some(unsafe {
            MemberCall::call_sret0::<i32>(function as *const c_void, std::ptr::with_exposed_provenance_mut(source))
        })
    }

    // The function in `slot` of the vtable at the start of `object`; `None` for a null vtable or function.
    fn virtual_fn(object: usize, slot: usize) -> Option<usize> {
        // SAFETY: `object` is the live block source of the local player; its first qword is its vtable pointer.
        let vtable = unsafe { std::ptr::with_exposed_provenance::<*const usize>(object).read() };
        if vtable.is_null() {
            return None;
        }
        // SAFETY: `vtable` is the vtable of the block source; the entry's recipe range keeps `slot` inside it.
        let function = unsafe { vtable.add(slot).read() };
        (function != 0).then_some(function)
    }

    fn lock_attach(&self) -> MutexGuard<'_, Option<BlockListenerAttach>> {
        self.attached.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn current(&self) -> Option<BlockListenerAttach> {
        *self.lock_attach()
    }
}
