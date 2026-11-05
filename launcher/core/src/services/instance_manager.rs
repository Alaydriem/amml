use crate::{
    AmmlInstaller, Compatibility, CoreProgress, HttpFetch, InstanceApi, InstanceGuard, InstanceHclFile, InstanceReader,
    InstanceState, LauncherBlock, LauncherBlockStore, LauncherError, LauncherPaths, ModCatalogue, ModChange, ModId,
    ModInstaller, ModSource, NewInstanceOutcome, NewInstanceRequest, Patcher, ResolvedMod, UpdateChecker, UpdateRow,
    UrlInspection, VersionJsonFile,
};
use amml_gdk::{
    Channel, GameVersion, InstallRequest, Installer, InstanceFolder, InstanceService, ProcessInspector, TreeOps,
    VersionStore,
};
use amml_registry::{AmmlRelease, RegistryRoot};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// Makes, updates and deletes instances, and installs mods into them.
pub struct InstanceManager<I: Installer, T: Patcher, P: ProcessInspector, H: HttpFetch> {
    paths: LauncherPaths,
    installer: I,
    instances: InstanceService,
    store: VersionStore,
    catalogue: ModCatalogue<H>,
    amml: AmmlInstaller<T, P>,
    mods: ModInstaller<P>,
    guard: InstanceGuard<P>,
}

impl<I: Installer, T: Patcher, P: ProcessInspector, H: HttpFetch> InstanceManager<I, T, P, H> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        paths: LauncherPaths,
        installer: I,
        instances: InstanceService,
        store: VersionStore,
        catalogue: ModCatalogue<H>,
        amml: AmmlInstaller<T, P>,
        mods: ModInstaller<P>,
        guard: InstanceGuard<P>,
    ) -> InstanceManager<I, T, P, H> {
        InstanceManager { paths, installer, instances, store, catalogue, amml, mods, guard }
    }

    pub fn catalogue_mut(&mut self) -> &mut ModCatalogue<H> {
        &mut self.catalogue
    }

    // Makes an instance: the game version (when it is not in the store yet), AMML, the core mods and the default mods.
    // A failure after the instance folder exists removes the folder again; the version store keeps what it got.
    pub fn create(
        &mut self,
        request: &NewInstanceRequest,
        progress: &mut impl FnMut(CoreProgress),
    ) -> Result<NewInstanceOutcome, LauncherError> {
        let folder_path = self.paths.checked_instance(&request.name)?;
        if folder_path.exists() {
            return Err(amml_gdk::GdkError::InstanceExists(request.name.clone()).into());
        }
        let (_, release) = self.pick_release(&request.game, request.amml.as_deref())?;

        let version_installed = self.store.get(&request.game).is_none();
        if version_installed {
            let channel = if request.preview { Channel::Preview } else { Channel::Release };
            let install = InstallRequest { version: request.game, channel };
            self.installer.install(&install, &mut |step, download| {
                progress(CoreProgress::InstallingVersion { step, download });
            })?;
        }

        progress(CoreProgress::CreatingInstance);
        let folder = self.instances.create(&request.game, &request.name)?;
        match self.populate(&folder, request, &release, progress) {
            Ok(skipped_mods) => {
                progress(CoreProgress::Done);
                let state = InstanceReader::read(&folder.path)?;
                Ok(NewInstanceOutcome { state, version_installed, skipped_mods })
            }
            Err(error) => {
                if let Err(cleanup) = TreeOps::remove(&folder.path) {
                    curia::error!("a failed instance was not removed", {
                        instance: request.name.clone(),
                        error: cleanup.to_string(),
                    });
                }
                Err(error)
            }
        }
    }

    // installs the newest AMML for the game (or `to`) over the one in the instance
    pub fn update_amml(
        &mut self,
        name: &str,
        to: Option<String>,
        progress: &mut impl FnMut(CoreProgress),
    ) -> Result<String, LauncherError> {
        let folder = self.instance_folder(name)?;
        self.guard.ensure_stopped(&folder, name)?;
        let state = InstanceReader::read(&folder)?;
        let (_, release) = self.pick_release(&state.game, to.as_deref())?;
        let zip = self.download_amml(&release, progress)?;
        let outcome = self.amml.install(&folder, name, &zip, progress);
        self.catalogue.discard(&zip);
        let version = outcome?;
        if folder.join(InstanceReader::VERSION_JSON).is_file() {
            LauncherBlockStore::set_amml(&folder, &version)?;
        }
        progress(CoreProgress::Done);
        Ok(version)
    }

    // Removes `instances\<name>` and nothing else; the shared data folder is never touched.
    pub fn delete(&self, name: &str, confirm: &str) -> Result<(), LauncherError> {
        let folder = self.instance_folder(name)?;
        if confirm != name {
            return Err(LauncherError::ConfirmMismatch { name: name.to_string() });
        }
        if !folder.is_dir() {
            return Err(LauncherError::NotAnInstance(folder));
        }
        self.guard.ensure_stopped(&folder, name)?;
        TreeOps::remove(&folder)?;
        Ok(())
    }

    pub fn install_mod(
        &mut self,
        name: &str,
        resolved: &ResolvedMod,
        progress: &mut impl FnMut(CoreProgress),
    ) -> Result<ModChange, LauncherError> {
        let folder = self.instance_folder(name)?;
        self.guard.ensure_stopped(&folder, name)?;
        let state = InstanceReader::read(&folder)?;
        let api = self.instance_api(&state)?;
        if resolved.release.api_version != api {
            return Err(LauncherError::ApiMismatch {
                id: resolved.document.id.clone(),
                mod_api: resolved.release.api_version,
                amml_api: api,
            });
        }
        let source = ModSource::Registry(resolved.registry.base.clone());
        self.install_resolved(&folder, name, resolved, source, api, progress)
    }

    // installs the zip of an inspected URL; a zip no registry lists needs the caller's consent
    pub fn install_inspected(
        &mut self,
        name: &str,
        inspection: UrlInspection,
        consent: bool,
    ) -> Result<ModChange, LauncherError> {
        let folder = self.instance_folder(name)?;
        let outcome = (|| {
            let source = inspection.source(consent)?;
            self.guard.ensure_stopped(&folder, name)?;
            let state = InstanceReader::read(&folder)?;
            let api = self.instance_api(&state)?;
            let core = inspection.matched.as_ref().map(|resolved| resolved.core);
            self.mods.install(&folder, name, &inspection.file, source, api, None, core)
        })();
        self.catalogue.discard(&inspection.file);
        outcome.map(|(_, change)| change)
    }

    // The release of a mod that fits the instance's AMML API. With a registry as `source`, only that registry is asked,
    // so an update comes from the registry the mod came from.
    pub fn resolve_mod(
        &mut self,
        name: &str,
        id: &ModId,
        source: Option<&ModSource>,
    ) -> Result<ResolvedMod, LauncherError> {
        let folder = self.instance_folder(name)?;
        let state = InstanceReader::read(&folder)?;
        let api = self.instance_api(&state)?;
        match source {
            Some(ModSource::Registry(base)) => self.catalogue.resolve_in(base, id, api),
            _ => self.catalogue.resolve(id, api),
        }
    }

    // Compares the instance with the registries; nothing is installed.
    pub fn check_updates(&mut self, name: &str) -> Result<Vec<UpdateRow>, LauncherError> {
        let folder = self.instance_folder(name)?;
        let state = InstanceReader::read(&folder)?;
        Ok(UpdateChecker::check(&mut self.catalogue, &state))
    }

    // Whether a mod is core. The official registry's document decides when it can be read (a cached copy counts). When
    // it cannot, the flag recorded at install time decides; a mod of the official registry with no record cannot be
    // judged, and that is an error, not a "no".
    fn is_core(&mut self, folder: &Path, id: &ModId) -> Result<bool, LauncherError> {
        let entry = LauncherBlockStore::read(folder).ok().and_then(|block| block.mods.get(&id.as_str()).cloned());
        let recorded = entry.as_ref().and_then(|entry| entry.core);
        let Some(client) = self.catalogue.registries_mut().official_mut() else {
            return Ok(recorded.unwrap_or(false));
        };
        let base = client.registry().base.clone();
        match client.document(id) {
            Ok(cached) => Ok(cached.value.core || recorded == Some(true)),
            Err(error) => match recorded {
                Some(core) => Ok(core),
                None if entry.is_some_and(|entry| entry.source == ModSource::Registry(base.clone())) => {
                    Err(match error {
                        offline @ LauncherError::Offline { .. } => offline,
                        _ => LauncherError::Offline { url: base },
                    })
                }
                None => Ok(false),
            },
        }
    }

    pub fn set_mod_enabled(&mut self, name: &str, id: &ModId, enabled: bool) -> Result<(), LauncherError> {
        let folder = self.instance_folder(name)?;
        let core = self.is_core(&folder, id)?;
        self.mods.set_enabled(&folder, name, id, enabled, core)
    }

    pub fn remove_mod(&mut self, name: &str, id: &ModId) -> Result<(), LauncherError> {
        let folder = self.instance_folder(name)?;
        let core = self.is_core(&folder, id)?;
        self.mods.remove(&folder, name, id, core)
    }

    // Installs each core mod of the official registry that the instance lacks; returns the ids installed.
    pub fn ensure_core_mods(
        &mut self,
        name: &str,
        progress: &mut impl FnMut(CoreProgress),
    ) -> Result<Vec<String>, LauncherError> {
        let folder = self.instance_folder(name)?;
        self.guard.ensure_stopped(&folder, name)?;
        let state = InstanceReader::read(&folder)?;
        let api = self.instance_api(&state)?;
        let mut installed = Vec::new();
        for resolved in self.catalogue.defaults_for(api)?.mods.into_iter().filter(|resolved| resolved.core) {
            if state.mods.iter().any(|held| held.id.as_str() == resolved.document.id) {
                continue;
            }
            let source = ModSource::Registry(resolved.registry.base.clone());
            self.install_resolved(&folder, name, &resolved, source, api, progress)?;
            installed.push(resolved.document.id.clone());
        }
        Ok(installed)
    }

    fn populate(
        &mut self,
        folder: &InstanceFolder,
        request: &NewInstanceRequest,
        release: &AmmlRelease,
        progress: &mut impl FnMut(CoreProgress),
    ) -> Result<Vec<(String, String)>, LauncherError> {
        let path = folder.path.clone();
        let mut version_json = VersionJsonFile::new_for(&request.name, &request.game, request.preview);
        version_json.set_launcher(&LauncherBlock::new(None, BTreeMap::new()));
        version_json.write(&path.join(InstanceReader::VERSION_JSON))?;

        let zip = self.download_amml(release, progress)?;
        let outcome = self.amml.install(&path, &request.name, &zip, progress);
        self.catalogue.discard(&zip);
        let version = outcome?;
        LauncherBlockStore::set_amml(&path, &version)?;
        InstanceHclFile::write(&InstanceReader::instance_hcl(&path), &request.data)?;

        let defaults = self.catalogue.defaults_for(release.api_version)?;
        let mut wanted = defaults.mods;
        // core mods first; the sort keeps the registry order inside each group
        wanted.sort_by_key(|resolved| !resolved.core);
        let mut skipped = defaults.skipped;
        for resolved in wanted {
            let source = ModSource::Registry(resolved.registry.base.clone());
            match self.install_resolved(&path, &request.name, &resolved, source, release.api_version, progress) {
                Ok(_) => {}
                Err(error) if !resolved.core => {
                    curia::warn!("a default mod was not installed", {
                        mod_id: resolved.document.id.clone(),
                        error: error.to_string(),
                    });
                    skipped.push((resolved.document.id.clone(), error.to_string()));
                }
                Err(error) => return Err(error),
            }
        }
        Ok(skipped)
    }

    fn install_resolved(
        &mut self,
        folder: &Path,
        name: &str,
        resolved: &ResolvedMod,
        source: ModSource,
        api: u32,
        progress: &mut impl FnMut(CoreProgress),
    ) -> Result<ModChange, LauncherError> {
        let id = resolved.document.id.clone();
        progress(CoreProgress::InstallingMod { id: id.clone() });
        let zip = self.catalogue.fetch(resolved, &mut |download| {
            progress(CoreProgress::Downloading { file: id.clone(), progress: download });
        })?;
        progress(CoreProgress::Verifying { file: id });
        let outcome = match ModId::parse(&resolved.document.id) {
            Ok(expected) => self.mods.install(folder, name, &zip, source, api, Some(&expected), Some(resolved.core)),
            Err(error) => Err(error),
        };
        self.catalogue.discard(&zip);
        outcome.map(|(_, change)| change)
    }

    fn download_amml(
        &mut self,
        release: &AmmlRelease,
        progress: &mut impl FnMut(CoreProgress),
    ) -> Result<crate::VerifiedFile, LauncherError> {
        let file = format!("amml-{}", release.version);
        let zip = self.catalogue.fetch_amml(release, &mut |download| {
            progress(CoreProgress::Downloading { file: file.clone(), progress: download });
        })?;
        progress(CoreProgress::Verifying { file });
        Ok(zip)
    }

    // The official registry's AMML release for the game: the requested version when given and listed, else the newest.
    fn pick_release(
        &mut self,
        game: &GameVersion,
        requested: Option<&str>,
    ) -> Result<(RegistryRoot, AmmlRelease), LauncherError> {
        let root = self.official_root()?;
        let release = match requested {
            Some(version) => root
                .amml
                .versions
                .iter()
                .find(|release| {
                    release.version == version
                        && release
                            .game_versions
                            .iter()
                            .filter_map(|entry| GameVersion::parse(entry).ok())
                            .any(|listed| listed == *game)
                })
                .cloned(),
            None => Compatibility::amml_for(game, &root),
        };
        match release {
            Some(release) => Ok((root, release)),
            None => Err(LauncherError::NotSupported { game: game.to_string() }),
        }
    }

    fn official_root(&mut self) -> Result<RegistryRoot, LauncherError> {
        let Some(client) = self.catalogue.registries_mut().official_mut() else {
            return Err(LauncherError::Offline { url: "the official registry".to_string() });
        };
        client.refresh();
        match client.root() {
            Some(root) => Ok(root.clone()),
            None => Err(LauncherError::Offline { url: client.registry().base.clone() }),
        }
    }

    // The API of the AMML in the instance: from the official registry's entry for its version, else from a mod in it.
    fn instance_api(&mut self, state: &InstanceState) -> Result<u32, LauncherError> {
        let root = self.official_root().ok();
        InstanceApi::of(state, root.as_ref()).ok_or_else(|| LauncherError::UnknownApi(state.name.clone()))
    }

    fn instance_folder(&self, name: &str) -> Result<PathBuf, LauncherError> {
        self.paths.checked_instance(name)
    }
}
