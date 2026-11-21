use std::ffi::c_void;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use amml_abi::{
    AmmlApi, AmmlBand, AmmlDirKind, AmmlPhase, AmmlStatus, AmmlStr, AmmlUseRow, EventCallbackFn, ModsLoaded,
    ProviderGone, ProviderReady,
};
use amml_data::{EntryId, EntryKind};
use amml_runtime::{EntryTable, ListenError, ListenerId, ProvideError, ProviderRegistry, ResourceStore, RestoreBook};

use crate::{
    ApiTable, AreaBook, CaptureService, CurrentMod, EventHub, GameHeap, HookEngine, InputService, LoaderConfig,
    LoaderDirs, LoaderError, LoaderLog, ModListener, ModPhase, ModRecord, OwnerEvents, ShutdownLatch, WindowService,
};

static STATE: OnceLock<LoaderState> = OnceLock::new();

/// Everything the loader holds for the life of the process: the hook engine, the resolved entries,
/// the game heap, the mod registry, the module base, and the v3 services: the event hub, the owned
/// resources, the provider registry, the input service and the flags to restore.
///
/// It lives in a `static` that is never dropped, which is what `HookEngine` requires. Mutation is
/// behind a `Mutex`; no lock is held while a mod callback or a hook installation touches the
/// other, so the two locks never nest.
pub struct LoaderState {
    engine: Mutex<HookEngine>,
    table: EntryTable,
    module_base: u64,
    heap: GameHeap,
    dirs: LoaderDirs,
    mods: Mutex<Vec<ModRecord>>,
    poison_logged: AtomicBool,
    api: AmmlApi,
    events: Arc<EventHub>,
    areas: Arc<AreaBook>,
    owner_events: Arc<OwnerEvents>,
    resources: Arc<ResourceStore<usize>>,
    input: InputService,
    window: WindowService,
    capture: CaptureService,
    shutdown: Arc<ShutdownLatch>,
    restores: RestoreBook,
    providers: ProviderRegistry<usize>,
    // Providers offered before `ModsLoaded`: owner, name, major, minor. Their `ProviderReady` follows `ModsLoaded`.
    pending_ready: Mutex<Vec<(usize, String, u16, u16)>>,
    mods_loaded: AtomicBool,
    // The instance folder and the redirected game data folder; empty when the host has none.
    instance_dir: String,
    game_data_dir: String,
}

impl LoaderState {
    /// Starts the loader state: opens the logs, builds the game heap from the table and the API
    /// table handed to mods. Works once per process.
    ///
    /// Not for use under the OS loader lock: it creates files and threads.
    ///
    /// # Safety
    /// The entries in `config.table` are values for the game module at `config.module_base`, so that
    /// `function` entries are code in this process once the base is added.
    pub unsafe fn init(config: LoaderConfig) -> Result<&'static LoaderState, LoaderError> {
        if STATE.get().is_some() {
            return Err(LoaderError::AlreadyInitialized);
        }
        LoaderLog::install(&config.dirs.logs)?;
        // SAFETY: the caller guarantees the table matches the module base.
        let heap = unsafe { GameHeap::from_table(&config.table, config.module_base) };
        let instance_dir = Self::folder_text(config.dirs.instance.as_deref());
        let game_data_dir = Self::folder_text(config.dirs.game_data.as_deref());
        let events = Arc::new(EventHub::new());
        let areas = Arc::new(AreaBook::new());
        let owner_events = Arc::new(OwnerEvents::new());
        let resources = Arc::new(ResourceStore::new());
        let input = InputService::new(events.clone(), areas.clone(), resources.clone(), owner_events.clone());
        let shutdown = Arc::new(ShutdownLatch::new());
        let window = WindowService::new(events.clone(), areas.clone(), shutdown.clone());
        let capture = CaptureService::new(resources.clone(), owner_events.clone(), shutdown.clone());
        let state = LoaderState {
            engine: Mutex::new(HookEngine::new(config.max_hooks)),
            table: config.table,
            module_base: config.module_base,
            heap,
            dirs: config.dirs,
            mods: Mutex::new(Vec::new()),
            poison_logged: AtomicBool::new(false),
            api: ApiTable::build(config.module_base),
            events,
            areas,
            owner_events,
            resources,
            input,
            window,
            capture,
            shutdown,
            restores: RestoreBook::new(),
            providers: ProviderRegistry::new(),
            pending_ready: Mutex::new(Vec::new()),
            mods_loaded: AtomicBool::new(false),
            instance_dir,
            game_data_dir,
        };
        STATE.set(state).map_err(|_| LoaderError::AlreadyInitialized)?;
        STATE.get().ok_or(LoaderError::AlreadyInitialized)
    }

    /// The state, once `init` ran.
    pub fn get() -> Option<&'static LoaderState> {
        STATE.get()
    }

    /// The table to give to mods. It lives as long as the process.
    pub fn api(&self) -> *const AmmlApi {
        &self.api
    }

    pub fn heap(&self) -> &GameHeap {
        &self.heap
    }

    pub fn dirs(&self) -> &LoaderDirs {
        &self.dirs
    }

    pub fn table(&self) -> &EntryTable {
        &self.table
    }

    pub fn module_base(&self) -> u64 {
        self.module_base
    }

    pub fn events(&self) -> &EventHub {
        &self.events
    }

    /// The areas the loader and its adapters offer.
    pub fn areas(&self) -> &AreaBook {
        &self.areas
    }

    /// The lease notices queued for mods, raised at the next frame.
    pub fn owner_events(&self) -> &OwnerEvents {
        &self.owner_events
    }

    /// The resources mods own, by id: input leases and holds (`InputService`) and capture stream ids
    /// (`CaptureService`), each with a plain value that needs no freeing on any thread.
    pub fn resources(&self) -> &ResourceStore<usize> {
        &self.resources
    }

    /// The input service: key table, leases, injection and the bound feeds.
    pub fn input(&self) -> &InputService {
        &self.input
    }

    /// The game window: the loader's subclass, its reads and the size requests of mods.
    pub fn window(&self) -> &WindowService {
        &self.window
    }

    /// Screenshots and capture streams.
    pub fn capture(&self) -> &CaptureService {
        &self.capture
    }

    /// The once-per-process shutdown, shared by the live shutdown at window close and the shutdown after the game
    /// stopped.
    pub fn shutdown(&self) -> &ShutdownLatch {
        &self.shutdown
    }

    /// The game flags mods set, with the player's own values to write back.
    pub fn restores(&self) -> &RestoreBook {
        &self.restores
    }

    /// The index of the mod `module` while it is loaded or enabled. An unknown module gives
    /// `INVALID_ARGUMENT`; a mod that is disabled, failed or unloaded gives `REFUSED`.
    pub(crate) fn live_mod(&self, module: &str) -> Result<usize, AmmlStatus> {
        let mods = self.lock_mods();
        let index = mods.iter().position(|record| record.id == module).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        match mods[index].phase {
            ModPhase::Loaded | ModPhase::Enabled => Ok(index),
            _ => Err(AmmlStatus::REFUSED),
        }
    }

    // Empty for no folder or a path that is not UTF-8.
    fn folder_text(path: Option<&Path>) -> String {
        path.and_then(Path::to_str).map(str::to_string).unwrap_or_default()
    }

    /// The mod records. A lock poisoned by a panic is recovered, because the records are plain
    /// data and the cleanup paths must still run; the first recovery is logged.
    pub(crate) fn lock_mods(&self) -> MutexGuard<'_, Vec<ModRecord>> {
        self.mods.lock().unwrap_or_else(|poisoned| {
            self.log_poison("mod registry");
            poisoned.into_inner()
        })
    }

    fn log_poison(&self, what: &str) {
        if !self.poison_logged.swap(true, Ordering::Relaxed) {
            curia::error!("a lock was poisoned by a panic and is recovered", { lock: what });
        }
    }

    /// An absolute address for `function` and `global` entries (`module_base + rva`, checked), the
    /// raw value for `vslot` and `field` entries. An id that is unknown or did not resolve is
    /// `MISSING_ENTRY`.
    pub fn resolve(&self, id: &str) -> Result<u64, AmmlStatus> {
        let resolved = self.table.get(&EntryId(id.to_string())).ok_or(AmmlStatus::MISSING_ENTRY)?;
        match resolved.kind {
            EntryKind::Function | EntryKind::Global => {
                self.module_base.checked_add(resolved.value).ok_or(AmmlStatus::FAILED)
            }
            EntryKind::VSlot | EntryKind::Field => Ok(resolved.value),
        }
    }

    /// Installs a hook for the mod whose callback is running. Hooks are accepted only while the
    /// loader is inside the mod's load or enable callback; any other caller, and the disable and
    /// unload callbacks, get `FAILED`. First hooks on a target patch its prologue and must happen
    /// before the game runs it.
    ///
    /// # Safety
    /// The contract of `HookEngine::hook`.
    pub(crate) unsafe fn install_hook(
        &self,
        target: *const c_void,
        detour: *const c_void,
        priority: i32,
        original: *mut *const c_void,
    ) -> Result<u64, AmmlStatus> {
        let Some(index) = CurrentMod::get().filter(|(_, callback)| callback.allows_hooks()).map(|(index, _)| index)
        else {
            curia::warn!("a hook was refused: hooks are installed only from a mod load or enable call");
            return Err(AmmlStatus::FAILED);
        };
        let handle = {
            let mut engine = self.engine.lock().map_err(|_| AmmlStatus::FAILED)?;
            // SAFETY: the caller upholds the contract of `HookEngine::hook`.
            unsafe { engine.hook(target, detour, priority, original) }.map_err(|error| {
                curia::warn!("a hook was refused", { error: error.to_string() });
                error.status()
            })?
        };
        if let Some(record) = self.lock_mods().get_mut(index) {
            record.hooks.push(handle);
        }
        Ok(handle)
    }

    /// Installs a hook that the loader itself owns (not a mod): it is not tied to a mod callback and
    /// no mod can remove it.
    ///
    /// # Safety
    /// The contract of `HookEngine::hook`.
    pub(crate) unsafe fn install_loader_hook(
        &self,
        target: *const c_void,
        detour: *const c_void,
        priority: i32,
        original: *mut *const c_void,
    ) -> Result<u64, AmmlStatus> {
        let mut engine = self.engine.lock().map_err(|_| AmmlStatus::FAILED)?;
        // SAFETY: the caller upholds the contract of `HookEngine::hook`.
        unsafe { engine.hook(target, detour, priority, original) }.map_err(|error| {
            curia::warn!("a loader hook was refused", { error: error.to_string() });
            error.status()
        })
    }

    /// Removes a hook for the mod whose callback is running. The mod has to hold the handle; a
    /// handle of another mod, an unknown handle and a call from outside a mod callback get `FAILED`.
    pub(crate) fn remove_hook(&self, handle: u64) -> Result<(), AmmlStatus> {
        let Some((index, _)) = CurrentMod::get() else {
            curia::warn!("an unhook was refused: hooks are removed only from a mod callback");
            return Err(AmmlStatus::FAILED);
        };
        let owned = self.lock_mods().get(index).is_some_and(|record| record.hooks.contains(&handle));
        if !owned {
            curia::warn!("an unhook was refused: the mod does not hold that hook", { handle: handle });
            return Err(AmmlStatus::FAILED);
        }
        {
            let mut engine = self.engine.lock().map_err(|_| AmmlStatus::FAILED)?;
            engine.unhook(handle).map_err(|error| error.status())?;
        }
        for record in self.lock_mods().iter_mut() {
            record.hooks.retain(|held| *held != handle);
        }
        Ok(())
    }

    /// Removes every hook the mod at `index` installed.
    pub(crate) fn remove_hooks_of(&self, index: usize) {
        let handles =
            self.lock_mods().get_mut(index).map(|record| std::mem::take(&mut record.hooks)).unwrap_or_default();
        let mut engine = self.engine.lock().unwrap_or_else(|poisoned| {
            self.log_poison("hook engine");
            poisoned.into_inner()
        });
        for handle in handles {
            if let Err(error) = engine.unhook(handle) {
                curia::warn!("a hook of a disabled mod could not be removed", { handle: handle, error: error.to_string() });
            }
        }
    }

    /// A view of one folder of a mod: `kind` 0 mod folder, 1 config folder, 2 data folder, 3 shared
    /// data folder (`INVALID_ARGUMENT` when the host has no shared data root). The bytes stay valid
    /// for the life of the process.
    pub(crate) fn mod_dir(&self, module: &str, kind: u8) -> Result<AmmlStr, AmmlStatus> {
        let index = usize::from(kind);
        if index > 3 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let mods = self.lock_mods();
        let record = mods.iter().find(|record| record.id == module).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        if record.dirs[index].is_empty() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        Ok(AmmlStr::from_str(&record.dirs[index]))
    }

    /// The mod whose load or enable callback runs on this thread; any other caller gets `FAILED`.
    fn registering_mod(call: &str) -> Result<usize, AmmlStatus> {
        match CurrentMod::get() {
            Some((index, callback)) if callback.allows_services() => Ok(index),
            _ => {
                curia::warn!("a call was refused: it is allowed only from a mod load or enable call", { call: call });
                Err(AmmlStatus::FAILED)
            }
        }
    }

    /// Registers a listener for the mod whose load or enable callback runs. An event that is not
    /// offered at `version` gives `NOT_AVAILABLE`; when the listen is `required`, the event is noted
    /// and the mod's callback then fails.
    pub(crate) fn listen(
        &self,
        event: &str,
        version: u32,
        band: AmmlBand,
        required: bool,
        callback: EventCallbackFn,
        user: usize,
    ) -> Result<u64, AmmlStatus> {
        let owner = Self::registering_mod("listen")?;
        let listener = ModListener { owner, callback, user };
        match self.events.listen(event, version, band, listener) {
            Ok(ListenerId(id)) => Ok(id),
            Err(error) => {
                let status = match error {
                    ListenError::UnknownEvent | ListenError::VersionTooNew => AmmlStatus::NOT_AVAILABLE,
                    ListenError::OwnerClosed => AmmlStatus::FAILED,
                };
                if status == AmmlStatus::NOT_AVAILABLE
                    && let Some(record) = self.lock_mods().get_mut(owner)
                {
                    if required {
                        record.missing_required.push(event.to_string());
                    } else {
                        Self::log_feature_off(record, event);
                    }
                }
                Err(status)
            }
        }
    }

    // An optional listen found its event not offered: the mod feature of the event's `uses` row is off.
    fn log_feature_off(record: &ModRecord, event: &str) {
        let feature = record
            .uses
            .iter()
            .find(|row| row.kind == AmmlUseRow::KIND_EVENT && row.name == event)
            .map_or("(not declared)", |row| row.feature.as_str());
        curia::warn!("feature off: its event is not available", {
            mod_name: record.id.as_str(),
            feature: feature,
            event: event
        });
    }

    /// Removes the listener behind `handle`; an unknown or already removed handle gives `INVALID_ARGUMENT`.
    pub(crate) fn unlisten(&self, handle: u64) -> Result<(), AmmlStatus> {
        if self.events.unlisten(ListenerId(handle)) { Ok(()) } else { Err(AmmlStatus::INVALID_ARGUMENT) }
    }

    /// Publishes `table` for the mod whose load or enable callback runs. Another mod holding the name
    /// and major gives `REFUSED`. `ProviderReady` follows `ModsLoaded`, or comes at once after it.
    pub(crate) fn provide(&self, name: &str, major: u16, minor: u16, table: usize) -> Result<(), AmmlStatus> {
        let owner = Self::registering_mod("provide")?;
        self.providers.provide(name, major, minor, owner, table).map_err(|ProvideError::Taken { owner: holder }| {
            curia::warn!("a provider was refused: another mod provides it", { name: name, major: major, holder: holder });
            AmmlStatus::REFUSED
        })?;
        if self.mods_loaded.load(Ordering::Acquire) {
            self.raise_provider_ready(name, major, minor);
        } else {
            self.lock_pending().push((owner, name.to_string(), major, minor));
        }
        Ok(())
    }

    /// The table of `name` at `major` with a minor of at least `min_minor`, with its minor, or `NOT_AVAILABLE`.
    pub fn require(&self, name: &str, major: u16, min_minor: u16) -> Result<(usize, u16), AmmlStatus> {
        self.providers.require(name, major, min_minor).ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    /// Raises `ModsLoaded` the first time it is called, then `ProviderReady` for every provider offered
    /// before it. Later calls do nothing.
    pub(crate) fn announce_mods_loaded(&self) {
        if self.mods_loaded.swap(true, Ordering::AcqRel) {
            return;
        }
        let mut loaded = ModsLoaded { header: EventHub::header::<ModsLoaded>(AmmlPhase::AFTER) };
        self.events.raise(&mut loaded, &self.events.loader_context());
        let pending = std::mem::take(&mut *self.lock_pending());
        for (_, name, major, minor) in pending {
            self.raise_provider_ready(&name, major, minor);
        }
    }

    /// Drops every provider of the mod at `owner` and raises `ProviderGone` to the other mods for each
    /// one, also for a provider whose `ProviderReady` has not gone out yet: a mod may have taken its
    /// table with `require` during its own load or enable. A mod that never saw the `ProviderReady`
    /// can ignore the `ProviderGone`.
    pub(crate) fn drop_providers_of(&self, owner: usize) {
        self.lock_pending().retain(|(holder, ..)| *holder != owner);
        for (name, major) in self.providers.drop_owner(owner) {
            let mut gone = ProviderGone {
                header: EventHub::header::<ProviderGone>(AmmlPhase::BEFORE),
                name: AmmlStr::from_str(&name),
                major,
            };
            self.events.raise_skipping(&mut gone, &self.events.loader_context(), Some(owner));
        }
    }

    fn raise_provider_ready(&self, name: &str, major: u16, minor: u16) {
        let mut ready = ProviderReady {
            header: EventHub::header::<ProviderReady>(AmmlPhase::AFTER),
            name: AmmlStr::from_str(name),
            major,
            minor,
        };
        self.events.raise(&mut ready, &self.events.loader_context());
    }

    fn lock_pending(&self) -> MutexGuard<'_, Vec<(usize, String, u16, u16)>> {
        self.pending_ready.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// A view of one folder of `module`: its mod, config, data or shared data folder, the instance
    /// folder or the redirected game data folder. An unknown module or kind gives `INVALID_ARGUMENT`;
    /// a folder the host does not have gives `NOT_AVAILABLE`. The bytes stay valid for the process.
    pub(crate) fn dir(&self, module: &str, kind: AmmlDirKind) -> Result<AmmlStr, AmmlStatus> {
        let mods = self.lock_mods();
        let record = mods.iter().find(|record| record.id == module).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        let folder = match kind {
            AmmlDirKind::MOD | AmmlDirKind::CONFIG | AmmlDirKind::DATA | AmmlDirKind::SHARED_DATA => {
                record.dirs[usize::from(kind.0)].as_str()
            }
            AmmlDirKind::INSTANCE => self.instance_dir.as_str(),
            AmmlDirKind::GAME_DATA => self.game_data_dir.as_str(),
            _ => return Err(AmmlStatus::INVALID_ARGUMENT),
        };
        if folder.is_empty() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(AmmlStr::from_str(folder))
    }
}
