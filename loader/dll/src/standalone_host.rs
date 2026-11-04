use amml_data::EntryId;
use amml_runtime::{CacheSource, DeriveSource, EntryTable, HostResolver};

use crate::{
    FolderRedirect, HeapCheck, HostError, HostInputs, HostPaths, HostProcess, HostStartup, InstanceSettings, LoaderLog,
};

/// The host where AMML starts itself, with no LeviLamina: the exe entry-point detour runs `run`
/// once, before the game's own start-up code.
///
/// Entries come from the cache file, then from a derive run on the exe (`HostResolver::standalone`);
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

    /// Resolves the entries of every mod under `paths.mods`, starts `LoaderState` over them and loads
    /// the mods. Works once per process, like `LoaderState`.
    pub fn start(paths: &HostPaths, module_base: u64) -> Result<(), HostError> {
        let (settings, scan) = HostStartup::prepare(paths)?;
        if settings.grade {
            curia::warn!("grade skipped: it compares with LeviLamina, which this host does not have");
        }
        let wanted = HostStartup::wanted(&scan.mods);
        let table = Self::resolve(paths, &wanted);
        HostStartup::log_coverage(&table, wanted.len());
        // SAFETY: the table holds RVAs from the cache or a derive run of the exe at `paths.exe`, which
        // is the running game module at `module_base`.
        unsafe { HostStartup::finish(paths, module_base, table, &scan, HeapCheck::Deferred) }
    }

    /// The table of `wanted` from the cache file under `paths.cache`, or from a derive run of
    /// `paths.exe` with the recipes under `paths.recipes` when the cache does not hold it. When the
    /// exe build or the recipes cannot be read, every entry is missing.
    pub fn resolve(paths: &HostPaths, wanted: &[EntryId]) -> EntryTable {
        let inputs = HostInputs::read(paths);
        let Some((build, digest)) = inputs.cache_key() else {
            return EntryTable::new(wanted);
        };
        let set = inputs.set().clone();
        let cache = CacheSource::new(paths.cache.clone(), *build, digest.clone(), set.clone());
        let derive = DeriveSource::new(paths.exe.clone(), paths.recipes.clone(), paths.cache.clone());
        HostResolver::standalone(cache, derive, set).resolve(wanted)
    }
}
