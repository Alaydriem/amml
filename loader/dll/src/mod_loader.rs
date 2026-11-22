use std::ffi::c_void;
use std::panic::AssertUnwindSafe;
use std::sync::LazyLock;
use std::time::Duration;

use amml_abi::{
    AMML_API_VERSION, AmmlEventContext, AmmlModInfo, AmmlPhase, AmmlStatus, AmmlStr, AmmlUseRow, ModId, Shutdown,
};
use amml_data::{AdapterCatalog, EntryId, NeedKind};
use amml_runtime::{AmmlVersion, ModRegistry, ModSpec};

use crate::{
    BlockEntityAdapter, BodyHideAdapter, Boundary, Callback, CameraAdapter, ClientAdapter, CurrentMod, EventHub,
    FreeCameraAdapter, InlineIconAdapter, ItemDrawAdapter, Library, LightingAdapter, LoaderLog, LoaderState,
    ModExports, ModLoadError, ModPhase, ModRecord, ModReport, NetworkAdapter, OptionsAdapter, RequestAdapter,
    SessionAdapter, ShapesAdapter, TextureAdapter, TimeAdapter, UseRow, WorldRenderAdapter,
};

// A workspace version that does not parse panics on first use; the package-version test catches this in CI.
#[allow(clippy::expect_used)]
static LOADER_AMML: LazyLock<AmmlVersion> = LazyLock::new(|| {
    AmmlVersion::parse(env!("CARGO_PKG_VERSION")).expect("the workspace version must look like X.Y.Z; fix Cargo.toml")
});

/// Loads, enables, disables and unloads mod DLLs.
///
/// Every call into a mod runs inside `Boundary::guard`, and a panic or a status other than `OK`
/// logs the failure, releases what the mod holds (listeners, resources, providers, hooks) and marks
/// it `Failed`; the next mod still runs.
/// A mod DLL is never freed: `unload_all` calls `amml_mod_unload` and the library stays mapped.
///
/// Call it after the host is past the OS loader lock: `LoadLibrary` runs the DLL entry point.
pub struct ModLoader {
    state: &'static LoaderState,
}

impl ModLoader {
    /// How long the live shutdown waits for the capture worker to write what it holds.
    pub const WRITER_WAIT: Duration = Duration::from_secs(2);

    pub fn new(state: &'static LoaderState) -> Self {
        Self { state }
    }

    /// The AMML release of this loader.
    pub fn amml_version() -> AmmlVersion {
        *LOADER_AMML
    }

    /// Maps each mod DLL and calls `amml_mod_load`, in the order of `specs`.
    pub fn load_all(&self, specs: &[ModSpec]) -> Vec<ModReport> {
        specs.iter().map(|spec| self.load_one(spec)).collect()
    }

    /// Calls `amml_mod_enable` on every loaded mod, in load order, then raises `ModsLoaded` (once per
    /// process) and the `ProviderReady` of every provider offered so far.
    ///
    /// A mod whose `uses` block names a required event the loader does not offer fails before its
    /// enable is called. A mod whose required listen found its event not offered fails, even when its
    /// enable returned `OK`.
    pub fn enable_all(&self) -> Vec<ModReport> {
        let mut reports = Vec::new();
        for (index, id) in self.names_in(ModPhase::Loaded) {
            self.state.events().reopen_owner(index);
            self.state.input().reopen_owner(index);
            let outcome = self
                .check_declared(index)
                .and_then(|()| self.call(index, Callback::Enable, |exports| exports.enable))
                .and_then(|()| self.check_required(index));
            let report = match outcome {
                Ok(()) => {
                    self.set_phase(index, ModPhase::Enabled);
                    FreeCameraAdapter::refresh_capture(self.state);
                    ModReport { id, phase: ModPhase::Enabled, error: None }
                }
                Err(error) => self.fail(index, id, ModPhase::Failed, error),
            };
            reports.push(report);
        }
        self.state.announce_mods_loaded();
        reports
    }

    /// Calls `amml_mod_disable` on every enabled mod, in reverse load order.
    pub fn disable_all(&self) {
        for (index, _) in self.names_in(ModPhase::Enabled).into_iter().rev() {
            self.disable_one(index);
        }
    }

    /// Disables the mod at `index` when it is enabled, in this order:
    /// 1. its providers are dropped and the other mods get `ProviderGone`, while the provider's table
    ///    is still live;
    /// 2. its listeners are removed and their calls on other threads are waited for, so no listener
    ///    of the mod runs from here on;
    /// 3. `amml_mod_disable` runs;
    /// 4. its input leases end (held keys and buttons get their ups at the next frame), the client flags
    ///    it was the last to set are queued for restore, its resources are forgotten and its hooks are
    ///    removed, whatever the call returned.
    ///
    /// The mod is `Disabling` meanwhile, so a second disable of it (from another thread, or from a
    /// listener the first one raised) does nothing. Not for use from inside a listener of that mod:
    /// step 2 would wait for the caller's own call.
    pub fn disable_one(&self, index: usize) {
        let Some(id) = self.begin_disable(index) else {
            return;
        };
        let game_alive = self.state.events().gate().is_open();
        curia::debug!("mod disable", { mod_name: id.as_str(), game_alive: game_alive });
        self.state.drop_providers_of(index);
        self.state.events().remove_owner(index);
        let outcome = self.call(index, Callback::Disable, |exports| exports.disable);
        self.release(index);
        match outcome {
            Ok(()) => self.set_phase(index, ModPhase::Disabled),
            Err(error) => {
                self.fail(index, id, ModPhase::Failed, error);
            }
        }
    }

    /// The process is ending and the game no longer runs: raises `Shutdown` with `game_alive` 0, closes
    /// the game gate and disables every mod. Runs once per process: after `shutdown_live` (or a first
    /// `shutdown`) it does nothing.
    pub fn shutdown(&self) {
        let Some(_run) = self.state.shutdown().run() else {
            curia::debug!("shutdown skipped: it ran already");
            return;
        };
        self.shutdown_stopped();
    }

    /// The game window closes while the game still runs; game thread only. In this order:
    /// 1. the shutdown latch closes (no size restore is posted, no screenshot or capture stream
    ///    is accepted from here on);
    /// 2. a joined world is left: `WorldLeft`, its resources taken back, the epoch 0;
    /// 3. the screenshot requests not copied yet fail, the capture worker gets up to
    ///    `WRITER_WAIT` to write the screenshots it holds, and the queued screenshot and lease
    ///    notices are raised, so each requester still gets both screenshot events;
    /// 4. `Shutdown` with `game_alive` 1 and `client` in the context;
    /// 5. every mod is disabled while the gate is open, so a mod may still call actions in its
    ///    listener and its `disable`;
    /// 6. the client flags of the disabled mods are written back to `client` and the key and
    ///    button ups of their leases reach the game;
    /// 7. the game gate closes for good, also when a step above panics.
    ///
    /// Runs once per process: a second call, or a later `shutdown`, does nothing. Before the gate
    /// opened it is `shutdown`: nothing of the game is called. The latch is finished when the run
    /// ends, also by a panic, so a waiting window thread is woken.
    pub fn shutdown_live(&self, client: usize) {
        let Some(_run) = self.state.shutdown().run() else {
            curia::debug!("shutdown skipped: it ran already");
            return;
        };
        let hub = self.state.events();
        if !hub.gate().is_open() {
            self.shutdown_stopped();
            return;
        }
        let ran = std::panic::catch_unwind(AssertUnwindSafe(|| self.run_live(client)));
        hub.gate().close();
        if let Err(panic) = ran {
            std::panic::resume_unwind(panic);
        }
    }

    // Steps 2 to 6 of `shutdown_live`.
    fn run_live(&self, client: usize) {
        let hub = self.state.events();
        curia::info!("shutdown", { game_alive: 1, client: client != 0 });
        SessionAdapter::with(SessionAdapter::leaving);
        let on_game_thread = hub.gate().on_game_thread();
        self.finish_screenshots(true, on_game_thread);
        let context =
            AmmlEventContext { client: std::ptr::with_exposed_provenance::<c_void>(client), ..hub.loader_context() };
        let mut shutdown = Shutdown { header: EventHub::header::<Shutdown>(AmmlPhase::BEFORE) };
        hub.raise(&mut shutdown, &context);
        self.disable_all();
        if on_game_thread {
            ClientAdapter::with((), |adapter| adapter.flush_restores(client));
            self.state.input().apply_frame(hub);
        } else {
            curia::warn!("restores not written: the shutdown does not run on the game thread");
        }
    }

    // Fails the screenshot requests not copied yet, waits up to `WRITER_WAIT` for the capture worker when `wait`,
    // then raises the queued owner notices when `raise`, while every mod still listens.
    fn finish_screenshots(&self, wait: bool, raise: bool) {
        let capture = self.state.capture();
        let failed = capture.fail_pending();
        if failed > 0 {
            curia::info!("screenshot requests not taken at shutdown", { count: failed });
        }
        if wait {
            let left = capture.wait_written(Self::WRITER_WAIT);
            if left > 0 {
                curia::warn!("capture worker still busy at shutdown, its jobs are lost", { left: left });
            }
        }
        if raise {
            self.state.owner_events().drain(self.state.events());
        }
    }

    // The screenshot requests not copied yet fail and, on the game thread, the queued notices are raised; then
    // `Shutdown` with `game_alive` 0, the gate closed, then every mod disabled.
    fn shutdown_stopped(&self) {
        let hub = self.state.events();
        curia::info!("shutdown", { game_alive: 0 });
        self.finish_screenshots(false, hub.gate().on_game_thread());
        let mut context = hub.loader_context();
        context.game_alive = 0;
        let mut shutdown = Shutdown { header: EventHub::header::<Shutdown>(AmmlPhase::BEFORE) };
        hub.raise(&mut shutdown, &context);
        hub.gate().close();
        self.disable_all();
    }

    /// Calls `amml_mod_unload` on every mod that was loaded, in reverse load order. The libraries stay mapped.
    pub fn unload_all(&self) {
        let phases = [
            ModPhase::Loaded,
            ModPhase::Enabled,
            ModPhase::Disabling,
            ModPhase::Disabled,
            ModPhase::Refused,
            ModPhase::Failed,
        ];
        let mut targets: Vec<(usize, String)> = Vec::new();
        for phase in phases {
            targets.extend(self.names_in(phase));
        }
        targets.sort();
        for (index, id) in targets.into_iter().rev() {
            self.release(index);
            if let Err(error) = self.call(index, Callback::Unload, |exports| exports.unload) {
                curia::warn!("a mod failed to unload", { mod_name: id, error: error.to_string() });
            }
            self.set_phase(index, ModPhase::Unloaded);
        }
    }

    /// The id and phase of every mod, in load order.
    pub fn phases(&self) -> Vec<(String, ModPhase)> {
        self.state.lock_mods().iter().map(|record| (record.id.clone(), record.phase)).collect()
    }

    fn load_one(&self, spec: &ModSpec) -> ModReport {
        LoaderLog::register_mod(&spec.id(), &spec.namespace, &spec.name);
        let outcome = self.try_load(spec);
        self.log_coverage(spec);
        match outcome {
            Ok(_) => {
                curia::info!("mod loaded", { mod_name: spec.id(), version: spec.version.clone().unwrap_or_default() });
                ModReport { id: spec.id(), phase: ModPhase::Loaded, error: None }
            }
            Err((Some(index), error)) => self.fail(index, spec.id(), Self::phase_for(&error), error),
            Err((None, error)) => {
                curia::error!("mod could not be loaded", { mod_name: spec.id(), error: error.to_string() });
                ModReport { id: spec.id(), phase: Self::phase_for(&error), error: Some(error) }
            }
        }
    }

    fn phase_for(error: &ModLoadError) -> ModPhase {
        match error {
            ModLoadError::VersionMismatch { .. } | ModLoadError::AmmlTooOld { .. } | ModLoadError::OlderMod { .. } => {
                ModPhase::Refused
            }
            ModLoadError::Call { status, .. } if *status == AmmlStatus::VERSION_MISMATCH => ModPhase::Refused,
            _ => ModPhase::Failed,
        }
    }

    // On failure the index is present once the mod has a record, so its hooks can be removed.
    fn try_load(&self, spec: &ModSpec) -> Result<usize, (Option<usize>, ModLoadError)> {
        if let Some(needs) = spec.amml_version
            && needs > *LOADER_AMML
        {
            return Err((None, ModLoadError::AmmlTooOld { needs, loader: *LOADER_AMML }));
        }
        if spec.api_version > AMML_API_VERSION {
            let error = ModLoadError::VersionMismatch { needs: spec.api_version, loader: AMML_API_VERSION };
            return Err((None, error));
        }
        let dirs = self.dirs_of(spec).ok_or((None, ModLoadError::PathNotUtf8))?;
        let path = spec.library_path();
        let library = Library::open(&path).map_err(|source| (None, ModLoadError::Library { path, source }))?;
        let exports = ModExports::find(&library).map_err(|error| (None, error))?;
        let index = {
            let mut mods = self.state.lock_mods();
            if mods.len() >= ModRegistry::MAX_MODS {
                return Err((None, ModLoadError::TooMany(mods.len())));
            }
            mods.push(ModRecord {
                id: spec.id(),
                _library: library,
                exports,
                phase: ModPhase::Loaded,
                hooks: Vec::new(),
                dirs: Box::new(dirs),
                uses: Vec::new(),
                missing_required: Vec::new(),
            });
            mods.len() - 1
        };
        let uses = self.call_load(index, spec).map_err(|error| (Some(index), error))?;
        if let Some(record) = self.state.lock_mods().get_mut(index) {
            record.uses = uses;
        }
        self.check_required(index).map_err(|error| (Some(index), error))?;
        Ok(index)
    }

    // Gives the `uses` rows of a v3 info block; a v2 block has none.
    fn call_load(&self, index: usize, spec: &ModSpec) -> Result<Vec<UseRow>, ModLoadError> {
        let mut info = AmmlModInfo {
            size: 0,
            api_version: 0,
            name: AmmlStr::EMPTY,
            namespace: AmmlStr::EMPTY,
            uses: std::ptr::null(),
        };
        let info_ptr = &raw mut info;
        let exports = self.exports_of(index)?;
        let api = self.state.api();
        let status = CurrentMod::with(index, Callback::Load, || {
            // SAFETY: the export has the `amml_mod_load` signature (`ModExports::find`); `api` lives
            // for the process and `info_ptr` is a writable AmmlModInfo that lives for the call.
            Boundary::guard(|| unsafe { (exports.load)(api, info_ptr) })
        });
        if !status.is_ok() {
            return Err(ModLoadError::Call { call: "amml_mod_load", status });
        }
        // SAFETY: `info_ptr` points to `info`, which is 8 aligned readable bytes at least.
        let Some((size, api_version)) = (unsafe { AmmlModInfo::header(info_ptr) }) else {
            return Err(ModLoadError::Call { call: "amml_mod_load", status: AmmlStatus::FAILED });
        };
        if size == 0 && api_version == 0 {
            return Err(ModLoadError::EmptyInfo);
        }
        if size < AmmlModInfo::SIZE_V2 {
            return Err(ModLoadError::OlderMod { found: api_version });
        }
        // SAFETY: `info` is initialized memory of the full v3 size; the header says the mod wrote at
        // least the v2 fields.
        let block = unsafe { info_ptr.read() };
        if !block.accepted_by(AMML_API_VERSION) {
            return Err(ModLoadError::VersionMismatch { needs: api_version, loader: AMML_API_VERSION });
        }
        // SAFETY: the mod keeps `name` and `namespace` valid until `amml_mod_unload` returns; they are read now.
        let (reported_namespace, reported_name) = unsafe { (block.namespace.as_str(), block.name.as_str()) };
        match (reported_namespace, reported_name) {
            (Ok(namespace), Ok(name)) if namespace == spec.namespace && name == spec.name => {}
            (namespace, name) => {
                return Err(ModLoadError::IdMismatch {
                    manifest: spec.id(),
                    reported: ModId::format(namespace.unwrap_or("?"), name.unwrap_or("?")),
                });
            }
        }
        // SAFETY: `info_ptr` points to `info`, whose `size` field the mod set; `uses` is read only when
        // `size` covers it.
        let uses = unsafe { AmmlModInfo::uses(info_ptr) };
        // SAFETY: a non-null `uses` from a v3 block points to the mod's `AmmlUses`, whose rows and
        // string views the mod keeps valid until `amml_mod_unload` returns.
        let rows = uses.map(|uses| unsafe { UseRow::read_all(uses) });
        Ok(rows.unwrap_or_default())
    }

    // A required listen made during the call that just ran found its event not offered: the call fails,
    // naming the feature of the mod's `uses` row for that event.
    fn check_required(&self, index: usize) -> Result<(), ModLoadError> {
        let mut mods = self.state.lock_mods();
        let Some(record) = mods.get_mut(index) else {
            return Ok(());
        };
        let Some(event) = std::mem::take(&mut record.missing_required).into_iter().next() else {
            return Ok(());
        };
        let feature = record
            .uses
            .iter()
            .find(|row| row.kind == AmmlUseRow::KIND_EVENT && row.name == event)
            .map_or_else(|| "(not declared)".to_string(), |row| row.feature.clone());
        Err(ModLoadError::RequiredEvent { feature, event })
    }

    // The first required event or area row of the mod's `uses` block that the loader does not offer
    // fails the enable, also when the mod would listen to it only later or never. An optional area row
    // that is not offered logs the feature that is off.
    fn check_declared(&self, index: usize) -> Result<(), ModLoadError> {
        let (id, rows) = {
            let mods = self.state.lock_mods();
            let Some(record) = mods.get(index) else {
                return Ok(());
            };
            (record.id.clone(), record.uses.clone())
        };
        for row in &rows {
            Self::note_too_old(&id, row, self.state);
            match row.kind {
                AmmlUseRow::KIND_EVENT if row.required && !self.state.events().is_available(&row.name, row.version) => {
                    return Err(ModLoadError::RequiredEvent { feature: row.feature.clone(), event: row.name.clone() });
                }
                AmmlUseRow::KIND_AREA if !self.state.areas().is_available(&row.name, row.version) => {
                    if !row.required {
                        curia::warn!("feature off: its area is not available", {
                            mod_name: id.as_str(),
                            feature: row.feature.as_str(),
                            area: row.name.as_str()
                        });
                        continue;
                    }
                    curia::warn!("a required area is not available", {
                        mod_name: id.as_str(),
                        feature: row.feature.as_str(),
                        area: row.name.as_str()
                    });
                    return Err(ModLoadError::RequiredArea { feature: row.feature.clone(), area: row.name.clone() });
                }
                _ => {}
            }
        }
        Ok(())
    }

    // Logs `loader too old` for a row that this loader's catalog does not serve at the row's version (or does not
    // know): a newer loader is needed, the game build is not the cause. The row is then not available.
    fn note_too_old(id: &str, row: &UseRow, state: &LoaderState) {
        let (kind, available) = match row.kind {
            AmmlUseRow::KIND_EVENT => (NeedKind::Event, state.events().is_available(&row.name, row.version)),
            AmmlUseRow::KIND_AREA => (NeedKind::Area, state.areas().is_available(&row.name, row.version)),
            _ => return,
        };
        if available {
            return;
        }
        let served = AdapterCatalog::served_version(kind, &row.name);
        if served.is_some_and(|served| served >= row.version) {
            return;
        }
        curia::warn!("loader too old", {
            mod_name: id,
            feature: row.feature.as_str(),
            name: row.name.as_str(),
            version: row.version,
            served: served.map_or_else(|| "none".to_string(), |served| served.to_string()),
            required: row.required
        });
    }

    // Drops the providers of the mod at `index` (raising `ProviderGone` to the other mods), removes its
    // listeners (waiting for calls in progress), ends its input leases (its held keys and buttons get
    // their ups at the next frame; the mod is told nothing, its listeners are gone), queues the client
    // flags it was the last to set for restore at the next client tick, forgets its resources and
    // closes its capture streams and drops its waiting screenshot requests, and removes its hooks. Running it again
    // finds nothing left.
    fn release(&self, index: usize) {
        self.state.drop_providers_of(index);
        self.state.events().remove_owner(index);
        self.state.input().end_owner(index);
        self.state.window().end_owner(index);
        self.state.capture().end_owner(index);
        ClientAdapter::queue_restore(index, self.state.restores().release_owner(index));
        // Leases, holds and stream ids are plain values, so there is nothing to free on any thread.
        let module = self.state.lock_mods().get(index).map(|record| record.id.clone()).unwrap_or_default();
        LightingAdapter::with((), |adapter| adapter.clear_owner(index, &module));
        TextureAdapter::with((), |adapter| adapter.disable_owner(index));
        ItemDrawAdapter::with((), |adapter| adapter.disable_owner(index));
        InlineIconAdapter::with((), |adapter| adapter.disable_owner(index));
        RequestAdapter::with((), |adapter| adapter.disable_owner(index));
        WorldRenderAdapter::with((), |adapter| adapter.disable_owner(index));
        BlockEntityAdapter::with((), |adapter| adapter.disable_owner(index));
        CameraAdapter::with((), |adapter| adapter.disable_owner(index));
        ShapesAdapter::with((), |adapter| adapter.disable_owner(index));
        FreeCameraAdapter::with((), |adapter| adapter.disable_owner(index));
        OptionsAdapter::with((), |adapter| adapter.disable_owner(index));
        NetworkAdapter::with((), |adapter| adapter.disable_owner(index));
        TimeAdapter::with((), |adapter| adapter.disable_owner(index));
        BodyHideAdapter::with((), |adapter| adapter.disable_owner(index));
        let _ = self.state.resources().take_owner(index);
        self.state.remove_hooks_of(index);
    }

    // Moves an enabled mod to `Disabling` under one lock and gives its id; any other phase gives `None`.
    fn begin_disable(&self, index: usize) -> Option<String> {
        let mut mods = self.state.lock_mods();
        let record = mods.get_mut(index).filter(|record| record.phase == ModPhase::Enabled)?;
        record.phase = ModPhase::Disabling;
        Some(record.id.clone())
    }

    fn call(
        &self,
        index: usize,
        callback: Callback,
        pick: impl FnOnce(&ModExports) -> unsafe extern "C" fn() -> AmmlStatus,
    ) -> Result<(), ModLoadError> {
        let function = pick(&self.exports_of(index)?);
        let status = CurrentMod::with(index, callback, || {
            // SAFETY: the export has the signature of its type (`ModExports::find`).
            Boundary::guard(|| unsafe { function() })
        });
        if status.is_ok() { Ok(()) } else { Err(ModLoadError::Call { call: callback.export_name(), status }) }
    }

    fn fail(&self, index: usize, id: String, phase: ModPhase, error: ModLoadError) -> ModReport {
        curia::error!("mod switched off", { mod_name: id.as_str(), error: error.to_string() });
        self.release(index);
        self.set_phase(index, phase);
        ModReport { id, phase, error: Some(error) }
    }

    fn exports_of(&self, index: usize) -> Result<ModExports, ModLoadError> {
        let failed = || ModLoadError::Call { call: "registry", status: AmmlStatus::FAILED };
        self.state.lock_mods().get(index).map(|record| record.exports).ok_or_else(failed)
    }

    fn set_phase(&self, index: usize, phase: ModPhase) {
        if let Some(record) = self.state.lock_mods().get_mut(index) {
            record.phase = phase;
        }
    }

    fn names_in(&self, phase: ModPhase) -> Vec<(usize, String)> {
        self.state
            .lock_mods()
            .iter()
            .enumerate()
            .filter(|(_, record)| record.phase == phase)
            .map(|(index, record)| (index, record.id.clone()))
            .collect()
    }

    fn dirs_of(&self, spec: &ModSpec) -> Option<[String; 4]> {
        Some([
            spec.dir.to_str()?.to_string(),
            spec.dir.join("config").to_str()?.to_string(),
            spec.dir.join("data").to_str()?.to_string(),
            self.shared_data_of(spec),
        ])
    }

    // Empty when the host has no shared data root, the mod is outside the mods folder, or the path is not UTF-8.
    fn shared_data_of(&self, spec: &ModSpec) -> String {
        let dirs = self.state.dirs();
        let shared = dirs.mod_data.as_ref().and_then(|root| {
            let relative = spec.dir.strip_prefix(&dirs.mods).ok()?;
            root.join(relative).to_str().map(str::to_string)
        });
        shared.unwrap_or_default()
    }

    // An entry an `entry_is` check refused is not for this build: it counts apart and is not missing, so a mod
    // that lists the entries of both layouts logs at INFO on a correct build.
    fn log_coverage(&self, spec: &ModSpec) {
        let table = self.state.table();
        let refused = spec.entries.iter().filter(|id| table.not_for_build().contains_key(*id)).count();
        let missing: Vec<&str> = spec
            .entries
            .iter()
            .filter(|id| table.get(id).is_none() && !table.not_for_build().contains_key(*id))
            .map(|EntryId(id)| id.as_str())
            .collect();
        if missing.is_empty() {
            curia::info!("mod entries", {
                mod_name: spec.id(),
                wanted: spec.entries.len(),
                missing: 0,
                not_for_this_build: refused
            });
        } else {
            curia::warn!("mod entries did not all resolve", {
                mod_name: spec.id(),
                wanted: spec.entries.len(),
                missing: missing,
                not_for_this_build: refused
            });
        }
    }
}
