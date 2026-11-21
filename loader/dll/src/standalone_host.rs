use std::collections::BTreeSet;

use amml_data::{AdapterCatalog, EntryId};
use amml_runtime::{CacheSource, DeriveSource, EntryTable, HostResolver, ModSpec};

use crate::{
    ActorAdapter, ChatAdapter, ClientAdapter, FolderRedirect, ForegroundHandback, HeapCheck, HostError, HostInputs,
    HostPaths, HostProcess, HostStartup, HudAdapter, InlineIconAdapter, InputAdapter, InstanceSettings,
    InventoryAdapter, ItemDrawAdapter, LevelRendererAdapter, LightingAdapter, LoaderLog, LocatorBarAdapter,
    NetworkAdapter, OverlayService, PlayerActionAdapter, RequestAdapter, RouterAdapter, ScreenAdapter, SessionActions,
    SessionAdapter, TestWorld, TextAdapter, TextureAdapter, TextureReloadAdapter, UiDrawAdapter, UiInputAdapter,
};

/// The host where AMML starts itself, with no LeviLamina: the exe entry-point detour runs `run`
/// once, before the game's own start-up code.
///
/// Entries come from the cache file and the shipped data file, then from a derive run on the exe (`HostResolver::standalone`);
/// there is no symbol lookup. The game heap check is deferred to the first allocation a mod makes,
/// because the game's static initializers have not run yet, and mods must not call game code in load
/// or enable.
pub struct StandaloneHost;

impl StandaloneHost {
    /// Installs the loader log under `<game>/amml/logs/`, starts the loader and enables the mods.
    /// Returns whether the loader started.
    pub fn run() -> bool {
        let Some(exe) = HostProcess::exe_path() else {
            return false;
        };
        let paths = HostPaths::standalone(&exe);
        if std::fs::create_dir_all(&paths.logs).is_err() || LoaderLog::install(&paths.logs).is_err() {
            return false;
        }
        let module_base = HostProcess::module_base();
        curia::info!("standalone host", {
            exe: paths.exe.display().to_string(),
            module_base: format!("{module_base:#x}")
        });
        // The redirect has no `Drop`: it stays in place for the life of the process.
        let _redirect = Self::install_redirect(&paths);
        match Self::start(&paths, module_base) {
            Ok(()) => HostStartup::enable(),
            Err(error) => {
                curia::error!("the loader did not start", { error: error.to_string() });
                false
            }
        }
    }

    /// Reads `amml/instance.hcl` and, when it sets `data_dir`, makes the game see that folder as its
    /// roaming data folder. No file, a file without `data_dir`, or any error gives `None` and the
    /// game keeps `%APPDATA%`; an error is logged and never stops the host.
    pub fn install_redirect(paths: &HostPaths) -> Option<FolderRedirect> {
        let (settings, unknown) = match InstanceSettings::load_with_unknown(&paths.instance, &paths.game) {
            Ok(loaded) => loaded,
            Err(error) => {
                curia::error!("instance file not accepted, the game keeps its data folder", {
                    error: error.to_string()
                });
                return None;
            }
        };
        for key in unknown {
            curia::warn!("unknown key in instance.hcl", { key: key });
        }
        let data_dir = settings.data_dir?;
        // SAFETY: the base is the running exe's own module, which stays mapped for the life of the process.
        let installed = unsafe { FolderRedirect::install(HostProcess::module_base() as usize, &data_dir) };
        match installed {
            Ok(redirect) => {
                curia::info!("data folder redirected", {
                    data_dir: data_dir.display().to_string(),
                    imports: redirect.slots().len()
                });
                Some(redirect)
            }
            Err(error) => {
                curia::error!("data folder redirect failed, the game keeps its data folder", {
                    data_dir: data_dir.display().to_string(),
                    error: error.to_string()
                });
                None
            }
        }
    }

    /// Resolves the entries of every mod under `paths.mods` and of the adapter catalog, starts
    /// `LoaderState` over them, installs the session, client and input adapters and the overlay, and loads the mods. Works once per process, like `LoaderState`.
    ///
    /// `AMML_TEST_WORLD` is read once here: when it names a level id, the test world entries are resolved
    /// too and the test world is installed, with a `test world requested` line.
    pub fn start(paths: &HostPaths, module_base: u64) -> Result<(), HostError> {
        let (settings, scan) = HostStartup::prepare(paths)?;
        if settings.grade {
            curia::warn!("grade skipped: it compares with LeviLamina, which this host does not have");
        }
        let test_world = TestWorld::from_env();
        ForegroundHandback::from_env().start();
        let mut wanted = Self::wanted(&scan.mods);
        if let Some(level_id) = test_world.level_id() {
            curia::info!("test world requested", { level_id: level_id, variable: TestWorld::VARIABLE });
            wanted.extend(TestWorld::ENTRIES.iter().map(|entry| EntryId((*entry).to_string())));
            wanted.sort();
            wanted.dedup();
        }
        let table = Self::resolve(paths, &wanted);
        HostStartup::log_coverage(&table, wanted.len());
        // SAFETY: the table holds RVAs from the cache or a derive run of the exe at `paths.exe`, which
        // is the running game module at `module_base`.
        unsafe {
            HostStartup::finish(paths, module_base, table, &scan, HeapCheck::Deferred, |state| {
                SessionAdapter::install(state);
                ClientAdapter::install(state);
                SessionActions::install(state);
                HudAdapter::install(state);
                ScreenAdapter::install(state);
                UiInputAdapter::install(state);
                UiDrawAdapter::install(state);
                TextAdapter::install(state);
                TextureAdapter::install(state);
                TextureReloadAdapter::install(state);
                ActorAdapter::install(state);
                InventoryAdapter::install(state);
                ItemDrawAdapter::install(state);
                RequestAdapter::install(state);
                NetworkAdapter::install(state);
                LocatorBarAdapter::install(state);
                ChatAdapter::install(state);
                InlineIconAdapter::install(state);
                LightingAdapter::install(state);
                PlayerActionAdapter::install(state);
                LevelRendererAdapter::install(state);
                RouterAdapter::install(state);
                InputAdapter::install(state);
                OverlayService::install(state);
                test_world.install();
            })
        }
    }

    /// The entries of `HostStartup::wanted` and those of every event and area in the adapter catalog (the
    /// required and the optional entries of each shape), which the adapters of this host hook and call; sorted
    /// and without repeats.
    pub fn wanted(mods: &[ModSpec]) -> Vec<EntryId> {
        let mut wanted: BTreeSet<EntryId> = HostStartup::wanted(mods).into_iter().collect();
        let adapters = AdapterCatalog::needs()
            .iter()
            .flat_map(|need| need.shapes)
            .flat_map(|shape| shape.entries.iter().chain(shape.optional));
        wanted.extend(adapters.map(|entry| EntryId((*entry).to_string())));
        wanted.into_iter().collect()
    }

    /// The table of `wanted` from the cache file under `paths.cache`, or from a derive run of
    /// the shipped data file under `paths.data`, or from a derive run of `paths.exe` with the recipes under
    /// `paths.recipes` for what neither holds. When the
    /// exe build or the recipes cannot be read, every entry is missing.
    pub fn resolve(paths: &HostPaths, wanted: &[EntryId]) -> EntryTable {
        let inputs = HostInputs::read(paths);
        let Some(build) = inputs.build() else {
            return EntryTable::new(wanted);
        };
        let set = inputs.set().clone();
        let cache = CacheSource::new(paths.cache.clone(), paths.data.clone(), build, set.clone());
        let derive =
            DeriveSource::new(paths.exe.clone(), paths.recipes.clone(), paths.cache.clone(), paths.data.clone());
        HostResolver::standalone(cache, derive, set).resolve(wanted)
    }
}
