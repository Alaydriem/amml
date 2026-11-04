use crate::{
    AmmlInstaller, AssetFetcher, DataChoice, GameInput, GameInputPackage, InstanceGuard, InstanceManager,
    LauncherError, LauncherPaths, LauncherSettings, LazyInstaller, ModCatalogue, ModInstaller, ProcessPatcher,
    RegistryCache, RegistryClient, RegistryRef, RegistrySet, SettingsStore, UreqHttpFetch,
};
use amml_gdk::{CancelFlag, Downloader, InstanceService, VersionStore, WindowsProcessInspector};
use std::sync::Arc;

pub type Manager = InstanceManager<LazyInstaller, ProcessPatcher, WindowsProcessInspector, UreqHttpFetch>;

// The one place that builds the catalogue and the instance manager from the launcher's folders and settings.
pub struct CoreServices {
    paths: LauncherPaths,
    official: Option<String>,
    cancel: CancelFlag,
}

impl CoreServices {
    // `official` replaces the base of the official registry; the settings file is never changed by it
    pub fn new(paths: LauncherPaths, official: Option<String>, cancel: CancelFlag) -> CoreServices {
        CoreServices { paths, official, cancel }
    }

    pub fn paths(&self) -> &LauncherPaths {
        &self.paths
    }

    pub fn settings(&self) -> Result<LauncherSettings, LauncherError> {
        let mut settings = SettingsStore::new(self.paths.clone()).load()?;
        if let Some(base) = &self.official
            && let Some(first) = settings.registries.first_mut()
        {
            *first = RegistryRef::official_at(base.clone());
        }
        Ok(settings)
    }

    pub fn data_choice(&self, own_data: bool, settings: &LauncherSettings) -> DataChoice {
        if own_data { DataChoice::Own } else { DataChoice::Shared(settings.data_dir(&self.paths)) }
    }

    // the flag every download of this launcher reads
    pub fn cancel(&self) -> &CancelFlag {
        &self.cancel
    }

    pub fn game_input(&self) -> GameInput {
        let downloads = self.paths.gdk().downloads();
        let downloader = Downloader::new(
            UreqHttpFetch::agent(),
            downloads.clone(),
            Downloader::DEFAULT_ATTEMPTS,
            self.cancel.clone(),
        );
        GameInput::new(AssetFetcher::new(downloader, downloads), self.cancel.clone(), GameInputPackage::official())
    }

    pub fn catalogue(&self) -> Result<ModCatalogue<UreqHttpFetch>, LauncherError> {
        let settings = self.settings()?;
        let clients = settings
            .registries
            .iter()
            .map(|registry| {
                RegistryClient::new(
                    registry.clone(),
                    UreqHttpFetch::new(UreqHttpFetch::agent()),
                    RegistryCache::new(self.paths.registry_cache(), RegistryCache::BUDGET),
                )
            })
            .collect();
        let downloads = self.paths.gdk().downloads();
        let downloader = Downloader::new(
            UreqHttpFetch::agent(),
            downloads.clone(),
            Downloader::DEFAULT_ATTEMPTS,
            self.cancel.clone(),
        );
        Ok(ModCatalogue::new(RegistrySet::new(clients), AssetFetcher::new(downloader, downloads)))
    }

    pub fn manager(&self) -> Result<Manager, LauncherError> {
        let gdk = self.paths.gdk().clone();
        let guard = InstanceGuard::new(Arc::new(WindowsProcessInspector::new()));
        Ok(InstanceManager::new(
            self.paths.clone(),
            LazyInstaller::new(gdk.clone(), self.cancel.clone()),
            InstanceService::new(gdk.clone(), VersionStore::new(gdk.clone())),
            VersionStore::new(gdk),
            self.catalogue()?,
            AmmlInstaller::new(ProcessPatcher::new(ProcessPatcher::DEFAULT_TIMEOUT), guard.clone()),
            ModInstaller::new(guard.clone()),
            guard,
        ))
    }
}
