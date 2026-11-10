use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlApi, AmmlStatus, AmmlStr};
use amml_data::{EntryId, EntryKind};
use amml_runtime::EntryTable;

use crate::{ApiTable, CurrentMod, GameHeap, HookEngine, LoaderConfig, LoaderDirs, LoaderError, LoaderLog, ModRecord};

static STATE: OnceLock<LoaderState> = OnceLock::new();

/// Everything the loader holds for the life of the process: the hook engine, the resolved entries,
/// the game heap, the mod registry and the module base.
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
        let state = LoaderState {
            engine: Mutex::new(HookEngine::new(config.max_hooks)),
            table: config.table,
            module_base: config.module_base,
            heap,
            dirs: config.dirs,
            mods: Mutex::new(Vec::new()),
            poison_logged: AtomicBool::new(false),
            api: ApiTable::build(config.module_base),
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
}
