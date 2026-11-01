use std::collections::BTreeSet;

use amml_data::EntryId;
use amml_runtime::{
    CacheSource, DeriveSource, EntryTable, Grade, HostResolver, ImageLayout, LeviLaminaSource, ModRegistry, ModScan,
    ModSpec, SymbolLookup,
};

use crate::{
    GameHeap, HeapCheck, HostError, HostInputs, HostPaths, HostProcess, LlSymbols, LoaderConfig, LoaderLog,
    LoaderSettings, LoaderState, ModLoader,
};

/// The steps of the host: what `ll_mod_load`, `ll_mod_enable`, `ll_mod_disable` and `ll_mod_unload`
/// do in the LeviLamina host.
///
/// Start-up has a part every host shares (`prepare`, `log_coverage`, `finish`) and a part that is
/// the host's own: where the paths come from, which sources resolve the entries, whether the
/// grade runs, and when the game heap is checked (`HeapCheck`).
pub struct HostStartup;

impl HostStartup {
    /// Every entry a mod lists, and the two the loader itself needs, sorted and without repeats.
    pub fn wanted(mods: &[ModSpec]) -> Vec<EntryId> {
        let mut wanted: BTreeSet<EntryId> = mods.iter().flat_map(|spec| spec.entries.iter().cloned()).collect();
        wanted.insert(EntryId(GameHeap::NEW_ID.to_string()));
        wanted.insert(EntryId(GameHeap::DELETE_ID.to_string()));
        wanted.into_iter().collect()
    }

    /// `ll_mod_load`: logging, symbol lookup, entry resolution, then the AMML mods. Returns whether
    /// the loader started; a mod that fails to load is logged and does not stop the others.
    pub fn load() -> bool {
        let Some(exe) = HostProcess::exe_path() else {
            return false;
        };
        let paths = HostPaths::levilamina(&exe);
        if std::fs::create_dir_all(&paths.logs).is_err() || LoaderLog::install(&paths.logs).is_err() {
            return false;
        }
        LlSymbols::install();
        let module_base = HostProcess::module_base();
        curia::info!("game module base", { module_base: format!("{module_base:#x}") });
        let layout = HostProcess::image_layout(module_base);
        if layout.size_of_image() == 0 {
            curia::warn!("the game image headers are unreadable; LeviLamina addresses are not used");
        }
        match Self::start(&paths, module_base, LlSymbols::lookup, layout) {
            Ok(()) => true,
            Err(error) => {
                curia::error!("the loader did not start", { error: error.to_string() });
                false
            }
        }
    }

    /// Creates the mods folder and the cache folder. A folder that cannot be created is not fatal:
    /// a missing mods folder scans as no mods, and a missing cache folder only costs the cache file.
    pub fn prepare_folders(paths: &HostPaths) -> Vec<HostError> {
        [&paths.mods, &paths.cache]
            .into_iter()
            .filter_map(|path| {
                std::fs::create_dir_all(path).err().map(|source| HostError::Folder { path: path.clone(), source })
            })
            .collect()
    }

    /// Shared by every host: reads and publishes the loader settings, creates the folders and scans
    /// the mods folder.
    pub fn prepare(paths: &HostPaths) -> Result<(LoaderSettings, ModScan), HostError> {
        let settings = LoaderSettings::load(&paths.settings)?;
        settings.publish();
        curia::info!("loader config", { grade: settings.grade });

        for error in Self::prepare_folders(paths) {
            curia::warn!("folder not created", { error: error.to_string() });
        }
        let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&paths.mods)?;
        for skipped in &scan.skipped {
            curia::warn!("mod folder skipped", {
                folder: skipped.dir.display().to_string(),
                reason: skipped.reason.as_str()
            });
        }
        Ok((settings, scan))
    }

    /// Shared by every host: starts `LoaderState` over `table`, runs the heap check when `heap_check`
    /// asks for it and loads the mods of `scan`.
    ///
    /// `LoaderState` starts once per process: a second start after `ll_mod_unload` finds the state in
    /// place, returns an error and loads nothing.
    ///
    /// # Safety
    /// `table` holds RVAs for the game module at `module_base`, so that `function` entries are code
    /// in this process once the base is added.
    pub unsafe fn finish(
        paths: &HostPaths,
        module_base: u64,
        table: EntryTable,
        scan: &ModScan,
        heap_check: HeapCheck,
    ) -> Result<(), HostError> {
        // SAFETY: the caller guarantees the table matches the module base.
        let state = unsafe { LoaderState::init(LoaderConfig::new(module_base, table, paths.loader_dirs())) }?;
        match heap_check {
            HeapCheck::AtLoad => state.heap().check(),
            HeapCheck::Deferred => {
                state.heap().defer_check();
                curia::info!("game heap round trip deferred to the first allocation: the game is not initialized yet");
            }
        }
        ModLoader::new(state).load_all(&scan.mods);
        Ok(())
    }

    /// The LeviLamina host: resolves the entries of every mod under `paths.mods` with LeviLamina's
    /// lookup first for functions, then starts `LoaderState` and loads the mods. The game heap is
    /// checked at load.
    ///
    /// `layout` is the game image at `module_base`; a LeviLamina address outside it is dropped.
    pub fn start(
        paths: &HostPaths,
        module_base: u64,
        lookup: SymbolLookup,
        layout: ImageLayout,
    ) -> Result<(), HostError> {
        let (settings, scan) = Self::prepare(paths)?;

        let inputs = HostInputs::read(paths);
        let set = inputs.set().clone();
        let levilamina = LeviLaminaSource::new(lookup, set.clone(), module_base, layout);
        let derived_sources = inputs.cache_key().map(|(build, digest)| {
            (
                *build,
                CacheSource::new(paths.cache.clone(), *build, digest.clone(), set.clone()),
                DeriveSource::new(paths.exe.clone(), paths.recipes.clone(), paths.cache.clone()),
            )
        });
        let resolver = match &derived_sources {
            Some((_, cache, derive)) => {
                HostResolver::levilamina(cache.clone(), derive.clone(), levilamina.clone(), set.clone())
            }
            None => HostResolver::levilamina_only(levilamina.clone(), set.clone()),
        };
        let wanted = Self::wanted(&scan.mods);
        let table = resolver.resolve(&wanted);
        Self::log_coverage(&table, wanted.len());
        if settings.grade {
            match derived_sources {
                Some((build, cache, derive)) => {
                    Self::grade(Grade::new(cache, derive, levilamina, set), paths, &build.file_stem())
                }
                None => curia::warn!("grade skipped: the exe build or the recipes cannot be read"),
            }
        }

        // SAFETY: the table holds RVAs for the game module at `module_base`, from the cache and derive
        // data of this exe build, or from LeviLamina's lookup of the running process, which accepts
        // only addresses inside the image.
        unsafe { Self::finish(paths, module_base, table, &scan, HeapCheck::AtLoad) }
    }

    /// `ll_mod_enable`: enables the loaded mods. Hooks are installed here.
    pub fn enable() -> bool {
        let Some(state) = LoaderState::get() else {
            return false;
        };
        ModLoader::new(state).enable_all();
        true
    }

    /// `ll_mod_disable`: disables the mods, which removes their hooks. With no state there is nothing to undo.
    pub fn disable() -> bool {
        let Some(state) = LoaderState::get() else {
            return true;
        };
        ModLoader::new(state).disable_all();
        true
    }

    /// `ll_mod_unload`: unloads the mods in reverse order. The libraries stay mapped. With no state there is nothing to undo.
    pub fn unload() -> bool {
        let Some(state) = LoaderState::get() else {
            return true;
        };
        ModLoader::new(state).unload_all();
        true
    }

    /// Writes `<logs>/grade-<stem>.md` and logs its summary line. A measurement only: the entry
    /// table the mods use is already built.
    fn grade(grade: Grade, paths: &HostPaths, stem: &str) {
        let (report, notes) = grade.run();
        for note in notes {
            curia::warn!("grade note", { note: note.to_string() });
        }
        match Grade::write(&report, &paths.logs, stem) {
            Ok(path) => curia::info!(report.summary_line().as_str(), { report: path.display().to_string() }),
            Err(error) => {
                curia::warn!(report.summary_line().as_str(), { error: error.to_string() });
            }
        }
    }

    /// Logs how many of the `wanted` entries resolved, which are missing and what the sources noted.
    ///
    /// An entry an `entry_is` check refused is not for this build (it belongs to the other layout of the game): it
    /// is logged at INFO, one line each, and counted apart, so the `resolved` count is over the entries that apply
    /// to this build and reads `resolved N/N` on a correct build.
    pub fn log_coverage(table: &EntryTable, wanted: usize) {
        let missing: Vec<String> = table.missing().into_iter().map(|EntryId(id)| id).collect();
        let refused = table.not_for_build();
        let applies = wanted.saturating_sub(refused.len());
        let line = format!(
            "resolved {}/{}; not for this build: {}; missing: {}",
            applies.saturating_sub(missing.len()),
            applies,
            refused.len(),
            missing.join(", ")
        );
        curia::info!(line.as_str(), { wanted: wanted, not_for_this_build: refused.len(), missing: missing.len() });
        for (EntryId(id), refusal) in refused {
            curia::info!("entry not for this build", { entry: id.as_str(), check: refusal.to_string() });
        }
        for note in table.notes() {
            curia::warn!("resolver note", { note: note.to_string() });
        }
    }
}
