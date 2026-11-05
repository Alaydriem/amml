use std::sync::LazyLock;

use amml_abi::{AMML_API_VERSION, AmmlModInfo, AmmlStatus, AmmlStr, ModId};
use amml_data::EntryId;
use amml_runtime::{AmmlVersion, ModRegistry, ModSpec};

use crate::{
    Boundary, Callback, CurrentMod, Library, LoaderLog, LoaderState, ModExports, ModLoadError, ModPhase, ModRecord,
    ModReport,
};

// A workspace version that does not parse panics on first use; the package-version test catches this in CI.
#[allow(clippy::expect_used)]
static LOADER_AMML: LazyLock<AmmlVersion> = LazyLock::new(|| {
    AmmlVersion::parse(env!("CARGO_PKG_VERSION")).expect("the workspace version must look like X.Y.Z; fix Cargo.toml")
});

/// Loads, enables, disables and unloads mod DLLs.
///
/// Every call into a mod runs inside `Boundary::guard`, and a panic or a status other than `OK`
/// logs the failure, removes the hooks of that mod and marks it `Failed`; the next mod still runs.
/// A mod DLL is never freed: `unload_all` calls `amml_mod_unload` and the library stays mapped.
///
/// Call it after the host is past the OS loader lock: `LoadLibrary` runs the DLL entry point.
pub struct ModLoader {
    state: &'static LoaderState,
}

impl ModLoader {
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

    /// Calls `amml_mod_enable` on every loaded mod, in load order.
    pub fn enable_all(&self) -> Vec<ModReport> {
        let mut reports = Vec::new();
        for (index, id) in self.names_in(ModPhase::Loaded) {
            let report = match self.call(index, Callback::Enable, |exports| exports.enable) {
                Ok(()) => {
                    self.set_phase(index, ModPhase::Enabled);
                    ModReport { id, phase: ModPhase::Enabled, error: None }
                }
                Err(error) => self.fail(index, id, ModPhase::Failed, error),
            };
            reports.push(report);
        }
        reports
    }

    /// Calls `amml_mod_disable` on every enabled mod, in reverse load order.
    pub fn disable_all(&self) {
        for (index, id) in self.names_in(ModPhase::Enabled).into_iter().rev() {
            let outcome = self.call(index, Callback::Disable, |exports| exports.disable);
            // A disabled mod's detours must stop running, whatever `amml_mod_disable` returned.
            self.state.remove_hooks_of(index);
            match outcome {
                Ok(()) => self.set_phase(index, ModPhase::Disabled),
                Err(error) => {
                    self.fail(index, id, ModPhase::Failed, error);
                }
            }
        }
    }

    /// Calls `amml_mod_unload` on every mod that was loaded, in reverse load order. The libraries stay mapped.
    pub fn unload_all(&self) {
        let phases = [ModPhase::Loaded, ModPhase::Enabled, ModPhase::Disabled, ModPhase::Refused, ModPhase::Failed];
        let mut targets: Vec<(usize, String)> = Vec::new();
        for phase in phases {
            targets.extend(self.names_in(phase));
        }
        targets.sort();
        for (index, id) in targets.into_iter().rev() {
            self.state.remove_hooks_of(index);
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
            });
            mods.len() - 1
        };
        self.call_load(index, spec).map_err(|error| (Some(index), error))?;
        Ok(index)
    }

    fn call_load(&self, index: usize, spec: &ModSpec) -> Result<(), ModLoadError> {
        let mut info = AmmlModInfo { size: 0, api_version: 0, name: AmmlStr::EMPTY, namespace: AmmlStr::EMPTY };
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
        // SAFETY: the header says the mod wrote a whole v2 block, and `info` is initialized memory.
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
        Ok(())
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
        self.state.remove_hooks_of(index);
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

    fn dirs_of(&self, spec: &ModSpec) -> Option<[String; 3]> {
        Some([
            spec.dir.to_str()?.to_string(),
            spec.dir.join("config").to_str()?.to_string(),
            spec.dir.join("data").to_str()?.to_string(),
        ])
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
