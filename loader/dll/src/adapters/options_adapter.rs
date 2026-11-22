use std::ffi::c_void;
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlOptionKey, AmmlPersistence, AmmlStatus, AmmlThread};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::{OptionRestore, TransientOptionBook, TransientOptionRefusal};

use crate::{ClientAdapter, GetIfValidFn, IntOptionLayout, LoaderState, OptionScan, SessionAdapter, ThreadNote};

static ADAPTER: OnceLock<OptionsAdapter> = OnceLock::new();

/// `int IOptionRegistry::getViewDistanceChunks() const`: rcx the registry, the chunks in eax.
type ViewDistanceFn = unsafe extern "C" fn(*mut c_void) -> i32;
/// `Option* IOptionRegistry::getEffectiveMaxFramerateOption()`: rcx the registry, rdx the 8-byte out pointer that
/// receives the `Option*`; rax the same out pointer.
type FramerateFn = unsafe extern "C" fn(*mut c_void, *mut usize) -> *mut usize;
/// `void IntOption::set(int value, bool saveOptionChange)`: rcx the option, edx the value, r8b the save flag.
type IntSetFn = unsafe extern "C" fn(*mut c_void, i32, u8);

/// The `options.view_distance` and `options.int` areas, over the option registry of the primary client.
///
/// Every call runs on the game thread (`REFUSED` elsewhere) with the game gate open (`NOT_READY`) and with the primary
/// client of the session adapter; the calls are not bound to a world. The view distance is the game's own
/// `getViewDistanceChunks`.
///
/// The int options are `VSYNC`, found by its save tag `gfx_vsync` with the scan of `OptionScan` (exactly one option
/// must carry it; a build where the scan fails does not offer `options.int`), and `MAX_FRAMERATE`, which is the option
/// `getEffectiveMaxFramerateOption` gives. That function gives option F6h or F7h depending on game state, so a hold keeps
/// the game option id of its first transient set: reads and a first set follow the effective option, later sets of the
/// holder and the restore follow the held option (through `getIfValid`). A value is read at `IntOptionLayout::VALUE_AT`, and written with the game's
/// `IntOption::set`: a transient set passes `saveOptionChange = false`, and the restore passes true.
///
/// One mod holds a transient value of a key (`TransientOptionBook`). The restore writes the value from before the first
/// transient set back only while the option still holds the transient value; a value the user set meanwhile is theirs.
/// A mod's disable restores its keys while the game runs and the disable runs on the game thread, and forgets them
/// otherwise.
///
/// The first read of each key in a world logs `option key` with the key, the option id and the save tag the Impl holds,
/// so the tag reading of `Option::Impl` at 188h can be checked in game.
pub struct OptionsAdapter {
    state: &'static LoaderState,
    options: Option<usize>,
    view_distance: Option<usize>,
    get_if_valid: Option<usize>,
    max_framerate: Option<usize>,
    int_set: Option<usize>,
    vsync_id: AtomicI32,
    book: Mutex<TransientOptionBook>,
    thread: ThreadNote,
    // For each key (vsync, max frame rate), the world epoch plus one in which its `option key` line was logged.
    logged: [AtomicU64; 2],
}

impl OptionsAdapter {
    pub const VIEW_DISTANCE: &'static str = "options.view_distance";
    pub const INT: &'static str = "options.int";
    /// The save tag of the vsync option.
    pub const VSYNC_TAG: &'static str = "gfx_vsync";
    const AREA_VERSION: u32 = 1;
    const UNKNOWN: i32 = -1;

    /// Offers each options area whose catalog shape resolved, and logs one `area adapter` / `area not available` line
    /// per row. Returns those rows. Works once per process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && (row.id == Self::VIEW_DISTANCE || row.id == Self::INT))
            .collect();
        let slot = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let adapter = Self {
            state,
            options: slot("ClientInstance.getOptions"),
            view_distance: slot("IOptionRegistry.getViewDistanceChunks"),
            get_if_valid: slot("OptionRegistry.getIfValid"),
            max_framerate: slot("OptionRegistry.getEffectiveMaxFramerateOption"),
            int_set: slot("IntOption.set"),
            vsync_id: AtomicI32::new(Self::UNKNOWN),
            book: Mutex::new(TransientOptionBook::new()),
            thread: ThreadNote::new(),
            logged: [AtomicU64::new(0), AtomicU64::new(0)],
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the options adapter is already installed");
            return rows;
        }
        for row in &rows {
            match row.shape {
                Some(shape) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                None => curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static OptionsAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&OptionsAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    // The book holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, TransientOptionBook> {
        self.book.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    // The game gate, the offer of `area`, the game thread (`NOT_READY`, `NOT_AVAILABLE`, `REFUSED`).
    fn enter(&self, area: &str) -> Result<(), AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(area, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let game = gate.on_game_thread();
        if self.thread.changed(if game { AmmlThread::GAME } else { AmmlThread::NETWORK }) {
            curia::info!("world thread", { hook: "ClientInstance.getOptions", game: game });
        }
        if !game {
            return Err(AmmlStatus::REFUSED);
        }
        Ok(())
    }

    // The option registry of `client`; null gives `NOT_AVAILABLE`.
    fn registry_of(&self, client: usize) -> Result<usize, AmmlStatus> {
        match ClientAdapter::call_pointer(client, self.options)? {
            0 => Err(AmmlStatus::NOT_AVAILABLE),
            registry => Ok(registry),
        }
    }

    // The registry of the primary client; `NOT_READY` while there is none.
    fn primary_registry(&self) -> Result<usize, AmmlStatus> {
        let client = SessionAdapter::primary_client().ok_or(AmmlStatus::NOT_READY)?;
        self.registry_of(client)
    }

    /// The view distance of `client` in chunks. `NOT_AVAILABLE` without the area, `NOT_READY` before the first frame,
    /// `REFUSED` off the game thread or for a client that is not the primary client, `FAILED` for a value that is not a
    /// distance.
    pub fn view_distance_chunks(&self, client: *const c_void) -> Result<u32, AmmlStatus> {
        self.enter(Self::VIEW_DISTANCE)?;
        let client = client.expose_provenance();
        if SessionAdapter::primary_client() != Some(client) {
            return Err(AmmlStatus::REFUSED);
        }
        let registry = self.registry_of(client)?;
        let function = ClientAdapter::virtual_fn(registry, self.view_distance).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `int IOptionRegistry::getViewDistanceChunks() const`, called on its own registry.
        let function = unsafe { std::mem::transmute::<usize, ViewDistanceFn>(function) };
        // SAFETY: `registry` is the live option registry of the primary client, on the game thread.
        let chunks = unsafe { function(std::ptr::with_exposed_provenance_mut(registry)) };
        u32::try_from(chunks).map_err(|_| AmmlStatus::FAILED)
    }

    /// The value of the int option `key`. `INVALID_ARGUMENT` for an unknown key; the other errors are those of the
    /// table function.
    pub fn get_int(&self, key: AmmlOptionKey) -> Result<i32, AmmlStatus> {
        Self::check_key(key)?;
        self.enter(Self::INT)?;
        let registry = self.primary_registry()?;
        let option = self.option_of(registry, key)?;
        // SAFETY: `option_of` gave a live int option.
        Ok(unsafe { IntOptionLayout::value(option) })
    }

    /// The mod at `owner` sets the int option `key` to `value`. Only `AmmlPersistence::TRANSIENT` exists:
    /// `INVALID_ARGUMENT` for another value. `REFUSED` when another mod holds a transient value of the key.
    pub fn set_int(
        &self,
        owner: usize,
        key: AmmlOptionKey,
        value: i32,
        persistence: AmmlPersistence,
    ) -> Result<(), AmmlStatus> {
        Self::check_key(key)?;
        if persistence != AmmlPersistence::TRANSIENT {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        self.enter(Self::INT)?;
        let registry = self.primary_registry()?;
        let set = self.int_set_fn()?;
        let mut book = self.lock();
        if book.holder(key.0).is_some_and(|holder| holder != owner) {
            return Err(AmmlStatus::REFUSED);
        }
        let option = match book.held_option(key.0) {
            Some(id) => self.held_option(registry, key, id)?,
            None => self.option_of(registry, key)?,
        };
        // SAFETY: `option_of` and `held_option` give a live int option with an Impl.
        let (id, current) = unsafe { (IntOptionLayout::id(option).unwrap_or(-1), IntOptionLayout::value(option)) };
        book.set(owner, key.0, id, current, value).map_err(|TransientOptionRefusal::Taken| AmmlStatus::REFUSED)?;
        // SAFETY: `option` is the live int option of the primary client, on the game thread; the save flag is false.
        unsafe { set(std::ptr::with_exposed_provenance_mut(option), value, 0) };
        curia::info!("option set", { key: Self::key_name(key), value: value, was: current, transient: true });
        Ok(())
    }

    /// The mod at `owner` ends its transient value of `key`. The errors are those of `AmmlOptionsTable::release_transient`.
    pub fn release_transient(&self, owner: usize, key: AmmlOptionKey) -> Result<(), AmmlStatus> {
        Self::check_key(key)?;
        self.enter(Self::INT)?;
        let registry = self.primary_registry()?;
        self.release_key(registry, owner, key)
    }

    // The restore step of one key: asks the book, and writes the value back when the option still holds the value the
    // mod set.
    fn release_key(&self, registry: usize, owner: usize, key: AmmlOptionKey) -> Result<(), AmmlStatus> {
        let mut book = self.lock();
        if book.holder(key.0).is_none() {
            return Ok(());
        }
        if book.holder(key.0) != Some(owner) {
            return Err(AmmlStatus::REFUSED);
        }
        let Some(id) = book.held_option(key.0) else {
            return Ok(());
        };
        let option = self.held_option(registry, key, id)?;
        let set = self.int_set_fn()?;
        // SAFETY: `held_option` gave a live int option.
        let current = unsafe { IntOptionLayout::value(option) };
        match book.release(owner, key.0, current).map_err(|TransientOptionRefusal::Taken| AmmlStatus::REFUSED)? {
            OptionRestore::Restore(value) => {
                // SAFETY: as in `set_int`; the restore saves the value.
                unsafe { set(std::ptr::with_exposed_provenance_mut(option), value, 1) };
                curia::info!("option restored", { key: Self::key_name(key), value: value });
            }
            OptionRestore::Keep => {
                curia::info!("option left", {
                    key: Self::key_name(key),
                    reason: "the value changed after the mod set it"
                });
            }
            OptionRestore::Nothing => {}
        }
        Ok(())
    }

    /// The mod at `owner` is disabled: its transient values are restored when the game runs, the disable runs on the
    /// game thread and a client exists; otherwise they are forgotten with no game call.
    pub fn disable_owner(&self, owner: usize) {
        let keys = self.lock().keys_of(owner);
        if keys.is_empty() {
            return;
        }
        let gate = self.state.events().gate();
        let registry = (gate.is_open() && gate.on_game_thread()).then(|| self.primary_registry().ok()).flatten();
        match registry {
            Some(registry) => {
                for key in keys {
                    if let Err(status) = self.release_key(registry, owner, AmmlOptionKey(key)) {
                        curia::warn!("a transient option value of a disabled mod was not restored", {
                            key: key,
                            status: status.0
                        });
                    }
                }
            }
            None => curia::info!("transient option values dropped", { count: keys.len(), reason: "no game call" }),
        }
        self.lock().forget_owner(owner);
    }

    fn check_key(key: AmmlOptionKey) -> Result<(), AmmlStatus> {
        if key == AmmlOptionKey::VSYNC || key == AmmlOptionKey::MAX_FRAMERATE {
            Ok(())
        } else {
            Err(AmmlStatus::INVALID_ARGUMENT)
        }
    }

    fn key_name(key: AmmlOptionKey) -> &'static str {
        if key == AmmlOptionKey::VSYNC { "vsync" } else { "max_framerate" }
    }

    fn int_set_fn(&self) -> Result<IntSetFn, AmmlStatus> {
        let address = self.int_set.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the `IntOption.set` entry is the address of `void IntOption::set(int, bool)`.
        Ok(unsafe { std::mem::transmute::<usize, IntSetFn>(address) })
    }

    // The live int option of `key` on `registry`. A build where the vsync scan fails, or where an option is not an int
    // option, does not offer `options.int` (the area is taken back, with a log line) and gives `NOT_AVAILABLE`.
    fn option_of(&self, registry: usize, key: AmmlOptionKey) -> Result<usize, AmmlStatus> {
        let option =
            if key == AmmlOptionKey::VSYNC { self.vsync_option(registry)? } else { self.framerate_option(registry)? };
        // SAFETY: `option` is a live option the registry gave.
        if !unsafe { IntOptionLayout::is_int(option) } {
            return Err(self.withdraw("an option is not an int option", Self::key_name(key), 0));
        }
        self.note_key(key, option);
        Ok(option)
    }

    fn vsync_option(&self, registry: usize) -> Result<usize, AmmlStatus> {
        let get = ClientAdapter::virtual_fn(registry, self.get_if_valid).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `getIfValid` of the registry.
        let get = unsafe { std::mem::transmute::<usize, GetIfValidFn>(get) };
        let mut id = self.vsync_id.load(Ordering::Acquire);
        if id == Self::UNKNOWN {
            // SAFETY: `registry` is the live option registry of the primary client, on the game thread, and `get` its
            // `getIfValid`.
            id = match unsafe { OptionScan::find(registry, get, Self::VSYNC_TAG) } {
                Ok(id) => id,
                Err(error) => {
                    return Err(self.withdraw("the vsync scan failed", Self::VSYNC_TAG, error.matches()));
                }
            };
            self.vsync_id.store(id, Ordering::Release);
        }
        // SAFETY: as above.
        unsafe { OptionScan::option(registry, get, id) }.ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    // The live int option with the game id `id` that a transient set of `key` wrote, through `getIfValid`; for the
    // restore and the later sets of a held key, which can name another game option by then (`MAX_FRAMERATE`).
    fn held_option(&self, registry: usize, key: AmmlOptionKey, id: i32) -> Result<usize, AmmlStatus> {
        let get = ClientAdapter::virtual_fn(registry, self.get_if_valid).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `getIfValid` of the registry.
        let get = unsafe { std::mem::transmute::<usize, GetIfValidFn>(get) };
        // SAFETY: `registry` is the live option registry of the primary client, on the game thread, and `get` its
        // `getIfValid`.
        let option = unsafe { OptionScan::option(registry, get, id) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: `option` is a live option the registry gave.
        if !unsafe { IntOptionLayout::is_int(option) } {
            return Err(self.withdraw("an option is not an int option", Self::key_name(key), 0));
        }
        Ok(option)
    }

    fn framerate_option(&self, registry: usize) -> Result<usize, AmmlStatus> {
        let function = ClientAdapter::virtual_fn(registry, self.max_framerate).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `getEffectiveMaxFramerateOption`, called on its own registry.
        let function = unsafe { std::mem::transmute::<usize, FramerateFn>(function) };
        let mut option = 0usize;
        // SAFETY: `registry` is the live option registry of the primary client, on the game thread; `option` has room for
        // the one pointer the function writes.
        unsafe { function(std::ptr::with_exposed_provenance_mut(registry), &raw mut option) };
        if option == 0 { Err(AmmlStatus::NOT_AVAILABLE) } else { Ok(option) }
    }

    // Takes `options.int` back and logs why; gives `NOT_AVAILABLE`.
    fn withdraw(&self, reason: &str, what: &str, matches: usize) -> AmmlStatus {
        curia::warn!("options.int not available", { reason: reason, option: what, matches: matches });
        self.state.areas().withdraw(Self::INT);
        AmmlStatus::NOT_AVAILABLE
    }

    // Logs `option key` with the id and the save tag the first time a key is read in each world.
    fn note_key(&self, key: AmmlOptionKey, option: usize) {
        let index = usize::from(key == AmmlOptionKey::MAX_FRAMERATE);
        let epoch = self.state.events().gate().epoch().0.wrapping_add(1);
        if self.logged[index].swap(epoch, Ordering::AcqRel) == epoch {
            return;
        }
        // SAFETY: `option` is a live int option with an Impl.
        let (id, tag) = unsafe { (IntOptionLayout::id(option).unwrap_or(-1), OptionScan::save_tag(option)) };
        curia::info!("option key", { key: Self::key_name(key), id: format!("{id:#x}"), save_tag: tag });
    }
}
