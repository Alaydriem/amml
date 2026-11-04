use super::session_state::SessionState;
use super::{
    CatalogService, DataStats, DownloadFolder, FaultMapper, InstallSteps, InstanceApi, JobRunner, JobSink, LastPlayed,
    LaunchGate, Locked, ManagerSlot, NameCheck, OpenPolicy, SetupState, SystemChecks, SystemInfo, VersionList,
    ViewMapper,
};
use crate::{
    AboutView, AdoptCandidatesView, AdoptView, BridgeError, CardView, CatalogLayoutView, CatalogSortView, ChannelView,
    CheckView, DataModeView, DataOverviewView, DownloadsView, FaultKindView, FaultView, InstanceView, JobError,
    JobKind, JobSnapshot, JobStateView, ModDetailView, ModView, NameProblemView, RegistryView, SearchGroupView,
    SettingsView, SetupView, UrlCheckView, VersionView, WorldLocationView, WorldSourceView,
};
use amml_gdk::{
    CancelFlag, CatalogueReader, Channel, DeveloperMode, DiskSpace, GameVersion, GdkError, Installer, JournalStore,
    PackageInventory, VersionStore, WindowsPackageInventory, WindowsProcessInspector,
};
use amml_launcher_core::{
    Adopter, CoreProgress, CoreServices, DataMigrator, InstanceGuard, InstanceReader, LauncherBlockStore,
    LauncherError, LazyInstaller, Manager, ModId, ModSource, NewInstanceRequest, ProcessGameStarter, RegistryCache,
    RegistryClient, RegistryRef, RegistrySet, RegistryStatus, SettingsStore, UpdateChecker, UpdateStatus,
    UreqHttpFetch, UrlInspector, WindowsWindowProbe,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::SystemTime;

// Everything the app does with the launcher core. Reads never wait on a job. Calls that need the manager answer
// `Busy` while a job holds it. Jobs run one at a time on the job thread.
pub struct LauncherSession<S: JobSink> {
    state: Arc<SessionState<S>>,
}

impl<S: JobSink> LauncherSession<S> {
    pub const OFFICIAL_AMML_ID: &'static str = UpdateChecker::AMML_ID;
    // the id of the "start from scratch" choice
    pub const SCRATCH: &'static str = "scratch";

    pub fn new(services: CoreServices, sink: S, cancel: CancelFlag) -> LauncherSession<S> {
        let paths = services.paths().clone();
        let guard = InstanceGuard::new(Arc::new(WindowsProcessInspector::new()));
        let state = SessionState {
            gate: LaunchGate::new(ProcessGameStarter::new(), WindowsWindowProbe::new(), guard, paths),
            runner: JobRunner::new(sink, cancel),
            start_guard: std::sync::Mutex::new(()),
            manager: ManagerSlot::new(),
            registries: std::sync::Mutex::new(Vec::new()),
            updates: std::sync::Mutex::new(BTreeMap::new()),
            startup: std::sync::Mutex::new(None),
            last_job: std::sync::Mutex::new(None),
            services,
        };
        LauncherSession { state: Arc::new(state) }
    }

    // the one job runner; tests start a job on it
    pub fn runner(&self) -> &JobRunner<S> {
        &self.state.runner
    }

    pub fn services(&self) -> &CoreServices {
        &self.state.services
    }

    // ---- reads: they never wait on a job ----

    pub fn instances(&self) -> Result<Vec<InstanceView>, BridgeError> {
        let services = &self.state.services;
        let settings = services.settings()?;
        let running = self.state.gate.running();
        let updates = Locked::of(&self.state.updates).clone();
        let mut views = Vec::new();
        for listed in InstanceReader::list(services.paths()) {
            match listed {
                Ok(state) => {
                    let count = updates.get(&state.name).copied().unwrap_or(0);
                    let played = LastPlayed::of(&state.folder);
                    views.push(ViewMapper::instance(&state, &settings, services.paths(), &running, count, played));
                }
                Err(error) => curia::debug!("a folder is not an instance", { error: error.to_string() }),
            }
        }
        Ok(views)
    }

    pub fn instance(&self, name: &str) -> Result<InstanceView, BridgeError> {
        let services = &self.state.services;
        let state = InstanceReader::read(&services.paths().checked_instance(name)?)?;
        let settings = services.settings()?;
        let count = Locked::of(&self.state.updates).get(name).copied().unwrap_or(0);
        let played = LastPlayed::of(&state.folder);
        Ok(ViewMapper::instance(&state, &settings, services.paths(), &self.state.gate.running(), count, played))
    }

    pub fn settings(&self) -> Result<SettingsView, BridgeError> {
        let services = &self.state.services;
        let settings = SettingsStore::new(services.paths().clone()).load()?;
        Ok(SettingsView {
            preview: settings.preview,
            data_dir: settings.data_dir(services.paths()).display().to_string(),
            logs_dir: services.paths().gdk().root().join("launcher").join("logs").display().to_string(),
        })
    }

    // Where the app starts. A PC that has AMML needs no first run; if it has no settings file yet, the defaults are
    // written and nothing else on disk changes: no instance is adopted and no instance file is touched.
    pub fn setup(&self) -> SetupView {
        let paths = self.state.services.paths();
        if SetupState::installed(paths) && !paths.settings().exists() {
            let store = SettingsStore::new(paths.clone());
            if let Err(error) = store.load().and_then(|defaults| store.save(&defaults)) {
                curia::error!("the default settings were not written", { error: error.to_string() });
            }
        }
        SetupState::view(paths)
    }

    pub fn checks(&self) -> Result<Vec<CheckView>, BridgeError> {
        let services = &self.state.services;
        let data_root = services.paths().gdk().root().to_path_buf();
        let free = DiskSpace::new().free_bytes(&data_root)?;
        let game_input = services.game_input().is_installed()?;
        Ok(SystemChecks::run(&WindowsPackageInventory::new(), game_input, free, &data_root)?)
    }

    // the versions in the version store; no network
    pub fn versions_on_disk(&self) -> Result<Vec<VersionView>, BridgeError> {
        let paths = self.state.services.paths();
        let folders = VersionStore::new(paths.gdk().clone()).list()?;
        let instances = Self::instance_states(&self.state);
        Ok(VersionList::on_disk(&folders, &instances))
    }

    pub fn adopt_candidates(&self) -> Result<AdoptCandidatesView, BridgeError> {
        let paths = self.state.services.paths();
        let instances = Adopter::candidates(paths)
            .into_iter()
            .map(|candidate| AdoptView {
                name: candidate.name,
                game: candidate.game.instance_name(),
                amml: candidate.amml,
                mod_count: candidate.mod_count as u32,
            })
            .collect();
        Ok(AdoptCandidatesView { instances, sources: self.world_sources() })
    }

    pub fn world_sources(&self) -> Vec<WorldSourceView> {
        let paths = self.state.services.paths();
        let Some(appdata) = DataMigrator::appdata_bedrock() else {
            return Vec::new();
        };
        DataMigrator::new(appdata, paths.clone())
            .candidates()
            .into_iter()
            .map(|source| WorldSourceView {
                id: source.label.clone(),
                label: source.label,
                path: source.path.display().to_string(),
                world_count: source.world_count as u32,
                modified: source.modified.map(super::UnixMillis::of),
            })
            .collect()
    }

    pub fn downloads(&self) -> Result<DownloadsView, BridgeError> {
        Ok(self.download_folder().view())
    }

    pub fn download_remove(&self, id: &str) -> Result<(), BridgeError> {
        self.state.unless_busy(|| self.download_folder().remove(id))
    }

    pub fn downloads_remove_unfinished(&self) -> Result<u32, BridgeError> {
        self.state.unless_busy(|| self.download_folder().remove_unfinished())
    }

    pub fn data_overview(&self) -> Result<DataOverviewView, BridgeError> {
        let services = &self.state.services;
        let paths = services.paths();
        let settings = services.settings()?;
        let folder = settings.data_dir(paths).join("Minecraft Bedrock");
        let figures = DataStats::of(&folder);
        let locations = InstanceReader::list(paths)
            .into_iter()
            .filter_map(Result::ok)
            .map(|state| {
                let own = state.data_dir.as_ref().filter(|dir| dir.starts_with(&state.folder));
                WorldLocationView {
                    instance: state.name,
                    mode: if own.is_some() { DataModeView::Own } else { DataModeView::Shared },
                    folder: own.map(|dir| dir.display().to_string()),
                }
            })
            .collect();
        Ok(DataOverviewView {
            folder: folder.display().to_string(),
            worlds: figures.worlds,
            resource_packs: figures.resource_packs,
            behavior_packs: figures.behavior_packs,
            bytes: figures.bytes,
            locations,
            sources: self.world_sources(),
        })
    }

    pub fn about(&self, launcher_version: &str, built_on: &str) -> Result<AboutView, BridgeError> {
        let services = &self.state.services;
        let store = WindowsPackageInventory::new().registered(Channel::Release.family_name()).ok().flatten();
        Ok(AboutView {
            launcher_version: launcher_version.to_string(),
            built_on: built_on.to_string(),
            windows_version: SystemInfo::windows_version(),
            store_package: store.as_ref().map(|package| package.full_name.clone()),
            store_version: store.as_ref().map(|package| package.version.to_string()),
            developer_mode: DeveloperMode::new().is_on().unwrap_or(false),
            registry: Locked::of(&self.state.registries).iter().find(|registry| registry.official).cloned(),
            logs_dir: services.paths().gdk().root().join("launcher").join("logs").display().to_string(),
        })
    }

    // The path to open when it lies inside the launcher's folder or the shared data folder.
    pub fn check_open_path(&self, path: &str) -> Result<std::path::PathBuf, BridgeError> {
        let services = &self.state.services;
        let roots = vec![services.paths().gdk().root(), services.settings()?.data_dir(services.paths())];
        OpenPolicy::path(&roots, path)
    }

    pub fn check_open_url(&self, url: &str) -> Result<String, BridgeError> {
        OpenPolicy::url(url)
    }

    pub fn name_problem(&self, name: &str) -> Option<NameProblemView> {
        let paths = self.state.services.paths();
        let taken: Vec<String> = std::fs::read_dir(paths.gdk().instances())
            .map(|entries| entries.flatten().map(|entry| entry.file_name().to_string_lossy().to_string()).collect())
            .unwrap_or_default();
        NameCheck::problem(name, &taken, paths)
    }

    pub fn job(&self) -> Option<JobSnapshot> {
        self.state.runner.snapshot()
    }

    pub fn cancel_job(&self) {
        self.state.runner.cancel();
    }

    pub fn startup_fault(&self) -> Option<FaultView> {
        Locked::of(&self.state.startup).clone()
    }

    // Finishes an install that was cut off. Call it once at start, off the window thread.
    pub fn recover_install(&self) -> Option<FaultView> {
        let services = &self.state.services;
        let installer = LazyInstaller::new(services.paths().gdk().clone(), services.cancel().clone());
        let fault = match installer.recover() {
            Ok(Some(version)) => {
                let values = BTreeMap::from([("version".to_string(), version)]);
                Some(FaultView::new(FaultKindView::InstallInterrupted, None, values))
            }
            Ok(None) => None,
            Err(error) => {
                curia::error!("the interrupted install was not finished", { error: error.to_string() });
                Some(FaultMapper::from_gdk(&error))
            }
        };
        *Locked::of(&self.state.startup) = fault.clone();
        fault
    }

    // ---- catalogue calls: they answer `Busy` while a job holds the manager ----

    // Asks every registry again and counts the updates waiting in each instance.
    pub fn refresh_registries(&self) -> Result<Vec<RegistryView>, BridgeError> {
        let paths = self.state.services.paths().clone();
        let views = self.state.try_manager(|manager| {
            manager.catalogue_mut().registries_mut().refresh_all();
            let now = SystemTime::now();
            let views: Vec<RegistryView> = manager
                .catalogue_mut()
                .registries_mut()
                .clients_mut()
                .iter()
                .map(|client| {
                    let fetched = matches!(client.status(), RegistryStatus::Ok).then_some(now);
                    ViewMapper::registry(client.registry(), client.root(), client.status(), None, fetched)
                })
                .collect();
            let mut counts = BTreeMap::new();
            for listed in InstanceReader::list(&paths) {
                let Ok(state) = listed else {
                    continue;
                };
                if !state.managed {
                    continue;
                }
                let rows = manager.check_updates(&state.name).unwrap_or_default();
                let waiting = rows.iter().filter(|row| row.status == UpdateStatus::Available).count();
                counts.insert(state.name, waiting as u32);
            }
            *Locked::of(&self.state.updates) = counts;
            Ok(views)
        })?;
        *Locked::of(&self.state.registries) = views.clone();
        Ok(views)
    }

    pub fn mods(&self, name: &str) -> Result<Vec<ModView>, BridgeError> {
        let paths = self.state.services.paths().clone();
        self.state.try_manager(|manager| {
            let folder = paths.checked_instance(name)?;
            let state = InstanceReader::read(&folder)?;
            let rows = manager.check_updates(name).unwrap_or_default();
            let core_ids: Vec<String> = LauncherBlockStore::read(&folder)
                .map(|block| {
                    block.mods.iter().filter(|(_, entry)| entry.core == Some(true)).map(|(id, _)| id.clone()).collect()
                })
                .unwrap_or_default();
            Ok(ViewMapper::mods(&state, &rows, &core_ids))
        })
    }

    // the versions of one channel; a catalogue the network cannot give is an error
    pub fn version_list(&self, channel: ChannelView) -> Result<Vec<VersionView>, BridgeError> {
        let paths = self.state.services.paths().clone();
        let wanted = match channel {
            ChannelView::Release => Channel::Release,
            ChannelView::Preview => Channel::Preview,
        };
        let (amml, defaults) = self.state.try_manager(|manager| {
            let Some(client) = manager.catalogue_mut().registries_mut().official_mut() else {
                return Ok((Vec::new(), 0));
            };
            client.refresh();
            let releases = client.root().map(|root| root.amml.versions.clone()).unwrap_or_default();
            let api = releases.iter().max_by_key(|release| release.api_version).map(|release| release.api_version);
            let defaults = api
                .and_then(|api| manager.catalogue_mut().defaults_for(api).ok())
                .map(|found| found.mods.len() as u32)
                .unwrap_or(0);
            Ok((releases, defaults))
        })?;
        let catalogue = CatalogueReader::fetch(&UreqHttpFetch::agent())?;
        let folders = VersionStore::new(paths.gdk().clone()).list()?;
        let instances = Self::instance_states(&self.state);
        let wanted_channel = match wanted {
            Channel::Release => ChannelView::Release,
            Channel::Preview => ChannelView::Preview,
        };
        Ok(VersionList::build(&catalogue, &folders, &amml, &instances, defaults)
            .into_iter()
            .filter(|view| view.channel == wanted_channel)
            .collect())
    }

    // The calls below take the instance the player is adding to. Its API and the mods it holds come from the
    // instance, so the caller names neither.

    pub fn catalog_layout(&self, name: &str, registry_id: &str) -> Result<CatalogLayoutView, BridgeError> {
        self.catalog(name, |set, api, _| CatalogService::layout(set, registry_id, api))
    }

    // `page` counts from 1
    pub fn catalog_page(&self, name: &str, registry_id: &str, page: u32) -> Result<Vec<CardView>, BridgeError> {
        self.catalog(name, |set, api, installed| CatalogService::page(set, registry_id, api, page, installed))
    }

    pub fn catalog_search(&self, name: &str, query: &str) -> Result<Vec<SearchGroupView>, BridgeError> {
        self.catalog(name, |set, api, installed| Ok(CatalogService::search(set, query, api, installed)))
    }

    pub fn catalog_sorted(
        &self,
        name: &str,
        registry_id: &str,
        sort: CatalogSortView,
    ) -> Result<Option<Vec<CardView>>, BridgeError> {
        self.catalog(name, |set, api, installed| CatalogService::sorted(set, registry_id, api, sort, installed))
    }

    pub fn catalog_detail(&self, name: &str, registry_id: &str, id: &str) -> Result<ModDetailView, BridgeError> {
        self.catalog(name, |set, api, _| CatalogService::detail(set, registry_id, id, api))
    }

    // Downloads the zip of a URL and says what it is. The file stays in the downloads folder for the install.
    pub fn inspect_url(&self, name: &str, url: &str) -> Result<UrlCheckView, BridgeError> {
        let paths = self.state.services.paths().clone();
        self.state.try_manager(|manager| {
            let state = InstanceReader::read(&paths.checked_instance(name)?)?;
            let root = Self::official_root(manager);
            let instance_api = InstanceApi::of(&state, root.as_ref());
            let inspection = UrlInspector::inspect(manager.catalogue_mut(), url, &mut |_| {})?;
            Ok(UrlCheckView {
                id: inspection.id.as_str(),
                title: inspection.listing.as_ref().map(|listing| listing.title.clone()),
                version: inspection.matched.as_ref().map(|resolved| resolved.release.version.clone()),
                api: inspection.api_version,
                api_matches: instance_api == Some(inspection.api_version),
                matched_registry: inspection.matched.as_ref().map(|resolved| resolved.registry.base.clone()),
                has_checksum: inspection.matched.is_some(),
            })
        })
    }

    fn catalog<R>(
        &self,
        name: &str,
        f: impl FnOnce(&mut RegistrySet<UreqHttpFetch>, u32, &[(String, Option<String>)]) -> Result<R, BridgeError>,
    ) -> Result<R, BridgeError> {
        let paths = self.state.services.paths().clone();
        self.state.try_manager(|manager| {
            let state = InstanceReader::read(&paths.checked_instance(name)?)?;
            let root = Self::official_root(manager);
            let api = InstanceApi::of(&state, root.as_ref())
                .ok_or_else(|| BridgeError::Launcher(LauncherError::UnknownApi(name.to_string())))?;
            let installed: Vec<(String, Option<String>)> =
                state.mods.iter().map(|held| (held.id.as_str(), held.version.clone())).collect();
            f(manager.catalogue_mut().registries_mut(), api, &installed)
        })
    }

    fn official_root(manager: &mut Manager) -> Option<amml_registry::RegistryRoot> {
        let client = manager.catalogue_mut().registries_mut().official_mut()?;
        if client.root().is_none() {
            client.refresh();
        }
        client.root().cloned()
    }

    // ---- direct calls: quick changes; they answer `Busy` while a job runs ----

    pub fn launch(&self, name: &str) -> Result<(), BridgeError> {
        // a job for this instance, or one for the whole app, may be writing its mods or its executable; while the ticket
        // lives, no job starts
        let _ticket = self.state.runner.begin_launch(name)?;
        let state = self.state.gate.launch(name)?;
        match state {
            amml_launcher_core::LaunchState::Exited { code, .. } => Err(BridgeError::GameExited(code)),
            _ => Ok(()),
        }
    }

    pub fn set_mod_enabled(&self, name: &str, id: &str, enabled: bool) -> Result<(), BridgeError> {
        let id = ModId::parse(id)?;
        self.state.try_manager(|manager| Ok(manager.set_mod_enabled(name, &id, enabled)?))
    }

    pub fn remove_mod(&self, name: &str, id: &str) -> Result<(), BridgeError> {
        let id = ModId::parse(id)?;
        self.state.try_manager(|manager| Ok(manager.remove_mod(name, &id)?))
    }

    pub fn delete(&self, name: &str, confirm: &str) -> Result<(), BridgeError> {
        self.state.try_manager(|manager| Ok(manager.delete(name, confirm)?))
    }

    pub fn remove_version(&self, version: &str) -> Result<bool, BridgeError> {
        let game = GameVersion::parse(version)?;
        self.state.unless_busy(|| Ok(VersionStore::new(self.state.services.paths().gdk().clone()).remove(&game)?))
    }

    // Adds a registry after asking it once; a registry that does not answer as one is refused.
    pub fn add_registry(&self, url: &str) -> Result<(), BridgeError> {
        let services = &self.state.services;
        let registry = RegistryRef::parse(url)?;
        // the check writes the registry cache, so it runs while the manager lock is held
        self.state.try_manager(|_| {
            let mut client = RegistryClient::new(
                registry.clone(),
                UreqHttpFetch::new(UreqHttpFetch::agent()),
                RegistryCache::new(services.paths().registry_cache(), RegistryCache::BUDGET),
            );
            match client.refresh() {
                RegistryStatus::Ok | RegistryStatus::Stale { .. } => Ok(()),
                RegistryStatus::Offline => Err(BridgeError::Unavailable(registry.base.clone())),
                RegistryStatus::Broken { reason } => Err(amml_launcher_core::LauncherError::BadSettings(reason).into()),
            }
        })?;
        // the settings are saved first; the manager is rebuilt from them, so a reader never rebuilds it from the old list
        self.state.unless_busy(|| {
            let store = SettingsStore::new(services.paths().clone());
            let mut settings = store.load()?;
            if !settings.registries.iter().any(|held| held.base == registry.base) {
                settings.registries.push(registry.clone());
                store.save(&settings)?;
            }
            self.state.reset_manager();
            Ok(())
        })
    }

    pub fn remove_registry(&self, base: &str) -> Result<(), BridgeError> {
        if base == amml_launcher_core::LauncherSettings::OFFICIAL_REGISTRY {
            return Err(BridgeError::Refused(base.to_string()));
        }
        self.state.unless_busy(|| {
            let store = SettingsStore::new(self.state.services.paths().clone());
            let mut settings = store.load()?;
            settings.registries.retain(|registry| registry.base != base);
            store.save(&settings)?;
            self.state.reset_manager();
            Ok(())
        })
    }

    pub fn set_preview(&self, on: bool) -> Result<(), BridgeError> {
        self.state.unless_busy(|| {
            let store = SettingsStore::new(self.state.services.paths().clone());
            let mut settings = store.load()?;
            settings.preview = on;
            Ok(store.save(&settings)?)
        })
    }

    // ---- jobs: one at a time ----

    pub fn create(&self, name: &str, game: &str, preview: bool, own_data: bool) -> Result<u64, BridgeError> {
        self.start(JobKind::CreateInstance { name: name.to_string(), game: game.to_string(), preview, own_data })
    }

    pub fn update_amml(&self, name: &str) -> Result<u64, BridgeError> {
        self.start(JobKind::UpdateAmml { name: name.to_string() })
    }

    pub fn install_mods(&self, name: &str, ids: Vec<String>) -> Result<u64, BridgeError> {
        self.start(JobKind::InstallMods { name: name.to_string(), ids })
    }

    pub fn install_url(&self, name: &str, url: &str, consent: bool) -> Result<u64, BridgeError> {
        self.start(JobKind::InstallUrl { name: name.to_string(), url: url.to_string(), consent })
    }

    // `ids` may hold the AMML row; it updates last
    pub fn update_mods(&self, name: &str, ids: Vec<String>) -> Result<u64, BridgeError> {
        self.start(JobKind::UpdateMods { name: name.to_string(), ids })
    }

    pub fn copy_worlds(&self, source: &str) -> Result<u64, BridgeError> {
        self.start(JobKind::CopyWorlds { source: source.to_string() })
    }

    pub fn install_game_input(&self) -> Result<u64, BridgeError> {
        self.start(JobKind::InstallGameInput)
    }

    // the last first-run step: the chosen worlds go into the shared data folder, then the settings are saved
    pub fn set_up_data(&self, source: &str) -> Result<u64, BridgeError> {
        self.start(JobKind::SetupData { source: source.to_string() })
    }

    // starts the last job again
    pub fn retry(&self) -> Result<u64, BridgeError> {
        let kind = Locked::of(&self.state.last_job).clone();
        match kind {
            Some(kind) => self.start(kind),
            None => Err(BridgeError::Unavailable("a task to repeat".to_string())),
        }
    }

    // Turns Developer Mode on through one UAC prompt, then runs the last job once more. Call it only after the
    // player said yes.
    pub fn retry_with_developer_mode(&self) -> Result<u64, BridgeError> {
        self.retry_after_consent(|| DeveloperMode::new().enable_with_consent())
    }

    // The job that failed is read before `enable` runs, and the one that starts is that job. If another job ran while
    // the player answered the prompt, nothing starts.
    pub fn retry_after_consent(&self, enable: impl FnOnce() -> Result<bool, GdkError>) -> Result<u64, BridgeError> {
        if self.state.runner.is_busy() {
            return Err(BridgeError::Busy);
        }
        // only after the exe copy failed and the install said it needs the player's yes
        let failed = self.state.runner.snapshot().filter(|snapshot| {
            snapshot.state == JobStateView::Failed
                && snapshot.fault.as_ref().is_some_and(|fault| fault.kind == FaultKindView::ConsentNeeded)
        });
        let Some(failed) = failed else {
            return Err(BridgeError::Refused("Developer Mode".to_string()));
        };
        let Some(kind) = Locked::of(&self.state.last_job).clone() else {
            return Err(BridgeError::Unavailable("a task to repeat".to_string()));
        };
        if !enable()? {
            return Err(BridgeError::Unavailable("Developer Mode".to_string()));
        }
        if self.state.runner.snapshot().map(|snapshot| snapshot.id) != Some(failed.id) {
            return Err(BridgeError::Busy);
        }
        self.start(kind)
    }

    // the downloads folder; the package of each registered Store version stays for a restore
    fn download_folder(&self) -> DownloadFolder {
        Self::download_folder_of(&self.state.services)
    }

    pub(crate) fn download_folder_of(services: &CoreServices) -> DownloadFolder {
        let inventory = WindowsPackageInventory::new();
        let mut kept: Vec<String> = [Channel::Release, Channel::Preview]
            .iter()
            .filter_map(|channel| inventory.registered(channel.family_name()).ok().flatten())
            .map(|package| format!("{}.msixvc", package.full_name))
            .collect();
        // the package a cut-off install puts back must stay while its journal exists
        if let Ok(Some(journal)) = JournalStore::new(services.paths().gdk().journal()).read()
            && let Some(name) = journal.original_package.file_name()
        {
            kept.push(name.to_string_lossy().to_string());
        }
        DownloadFolder::new(services.paths().gdk().downloads(), kept)
    }

    fn instance_states(state: &SessionState<S>) -> Vec<amml_launcher_core::InstanceState> {
        InstanceReader::list(state.services.paths()).into_iter().filter_map(Result::ok).collect()
    }

    // Builds the steps and the work of a job and hands them to the runner.
    fn start(&self, kind: JobKind) -> Result<u64, BridgeError> {
        let (steps, work) = self.plan(&kind)?;
        let state = Arc::clone(&self.state);
        // the busy flag is set under the guard, so a call that checked for a job before it cannot act after it
        let _guard = Locked::of(&self.state.start_guard);
        let id = self.state.runner.start(kind.clone(), steps, move |progress| state.run_job(progress, work))?;
        *Locked::of(&self.state.last_job) = Some(kind);
        Ok(id)
    }

    // Checks what can be checked before a job starts, so a bad request changes nothing.
    fn plan(&self, kind: &JobKind) -> Result<(InstallSteps, JobWork), BridgeError> {
        let services = &self.state.services;
        let paths = services.paths().clone();
        match kind.clone() {
            JobKind::CreateInstance { name, game, preview, own_data } => {
                if let Some(problem) = self.name_problem(&name) {
                    return Err(BridgeError::NameProblem(problem));
                }
                let game = GameVersion::parse(&game)?;
                let needed = VersionStore::new(paths.gdk().clone()).get(&game).is_none();
                let settings = services.settings()?;
                let request = NewInstanceRequest {
                    game,
                    preview,
                    name,
                    amml: None,
                    data: services.data_choice(own_data, &settings),
                };
                let work: JobWork = Box::new(move |manager, progress| {
                    let outcome = manager.create(&request, &mut |event| progress(event))?;
                    for (id, reason) in &outcome.skipped_mods {
                        curia::warn!("a default mod was skipped", { mod_id: id.clone(), reason: reason.clone() });
                    }
                    Ok(())
                });
                Ok((InstallSteps::new(needed, Vec::new()), work))
            }
            JobKind::UpdateAmml { name } => {
                let work: JobWork = Box::new(move |manager, progress| {
                    manager.update_amml(&name, None, &mut |event| progress(event))?;
                    Ok(())
                });
                Ok((InstallSteps::for_amml(), work))
            }
            JobKind::InstallMods { name, ids } => {
                let parsed = ids.iter().map(|id| ModId::parse(id)).collect::<Result<Vec<_>, _>>()?;
                let work: JobWork = Box::new(move |manager, progress| {
                    for id in &parsed {
                        let resolved = manager.resolve_mod(&name, id, None)?;
                        manager.install_mod(&name, &resolved, &mut |event| progress(event))?;
                    }
                    Ok(())
                });
                Ok((InstallSteps::for_mods(ids), work))
            }
            JobKind::UpdateMods { name, ids } => {
                let with_amml = ids.iter().any(|id| id == UpdateChecker::AMML_ID);
                let mod_ids: Vec<String> = ids.into_iter().filter(|id| id != UpdateChecker::AMML_ID).collect();
                let parsed = mod_ids.iter().map(|id| ModId::parse(id)).collect::<Result<Vec<_>, _>>()?;
                let folder = paths.checked_instance(&name)?;
                let work: JobWork = Box::new(move |manager, progress| {
                    let state = InstanceReader::read(&folder)?;
                    for id in &parsed {
                        let source: Option<ModSource> =
                            state.mods.iter().find(|held| &held.id == id).map(|held| held.source.clone());
                        let resolved = manager.resolve_mod(&name, id, source.as_ref())?;
                        manager.install_mod(&name, &resolved, &mut |event| progress(event))?;
                    }
                    if with_amml {
                        manager.update_amml(&name, None, &mut |event| progress(event))?;
                    }
                    Ok(())
                });
                Ok((InstallSteps::for_updates(mod_ids, with_amml), work))
            }
            JobKind::InstallUrl { name, url, consent } => {
                let work: JobWork = Box::new(move |manager, progress| {
                    let inspection = UrlInspector::inspect(manager.catalogue_mut(), &url, &mut |download| {
                        progress(CoreProgress::Downloading { file: "mod.zip".to_string(), progress: download });
                    })?;
                    manager.install_inspected(&name, inspection, consent)?;
                    Ok(())
                });
                Ok((InstallSteps::single("mod-url"), work))
            }
            JobKind::CopyWorlds { source } => {
                let from = self.world_source_path(&source)?;
                let data_dir = services.settings()?.data_dir(&paths);
                let migrator = DataMigrator::new(DataMigrator::appdata_bedrock().unwrap_or_default(), paths);
                let work: JobWork = Box::new(move |_, _| {
                    migrator.copy(&from, &data_dir, &mut |_| {})?;
                    Ok(())
                });
                Ok((InstallSteps::single("worlds"), work))
            }
            JobKind::InstallGameInput => {
                let input = services.game_input();
                let work: JobWork = Box::new(move |_, progress| {
                    if !input.install(&mut |event| progress(event))? {
                        curia::warn!("GameInput is still missing after the install");
                    }
                    Ok(())
                });
                Ok((InstallSteps::single("game-input"), work))
            }
            JobKind::SetupData { source } => {
                let from = if source == Self::SCRATCH { None } else { Some(self.world_source_path(&source)?) };
                let data_dir = services.settings()?.data_dir(&paths);
                let migrator = DataMigrator::new(DataMigrator::appdata_bedrock().unwrap_or_default(), paths.clone());
                let work: JobWork = Box::new(move |_, _| {
                    // the copy comes first: a copy that fails leaves no settings file, so the first run shows again
                    if let Some(from) = &from {
                        migrator.copy(from, &data_dir, &mut |_| {})?;
                    }
                    let store = SettingsStore::new(paths.clone());
                    store.save(&store.load()?)?;
                    Ok(())
                });
                Ok((InstallSteps::single("setup-data"), work))
            }
        }
    }

    fn world_source_path(&self, id: &str) -> Result<std::path::PathBuf, BridgeError> {
        self.world_sources()
            .into_iter()
            .find(|source| source.id == id)
            .map(|source| std::path::PathBuf::from(source.path))
            .ok_or_else(|| BridgeError::Unavailable(id.to_string()))
    }
}

// the work of one job: it gets the manager and the progress callback
type JobWork = Box<dyn FnOnce(&mut Manager, &mut dyn FnMut(CoreProgress)) -> Result<(), JobError> + Send>;
