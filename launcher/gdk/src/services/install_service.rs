use super::{
    DownloadCache, Downloader, ExeCopier, InstallLock, Installer, JournalStore, PackageDeployer, PackageInventory,
    ProcessInspector, StoreCheck, TreeOps, VersionStore,
};
use crate::GdkError;
use crate::structs::{
    AmmlPaths, Catalogue, Channel, DownloadProgress, GameVersion, InstallJournal, InstallOutcome, InstallRequest,
    InstallStep, RegisteredPackage, SpaceNeed, StoreState, VersionFolder,
};
use std::path::Path;
use std::time::Duration;

// Puts one game version into the version store.
//
// The registered version is copied out without any registration change. Any other version is installed by
// Windows from the player's licence, copied out, removed again, and the registered version is restored from a
// package that was downloaded before the first change. A journal on disk lets a later run finish the restore
// when this process is stopped half way.
//
// The service decides which packages the deployer must never remove: the journal's original package and the
// package that was registered when the install started.
pub struct InstallService<I, P, I2, D, E> {
    paths: AmmlPaths,
    catalogue: Catalogue,
    downloader: Downloader,
    cache: DownloadCache,
    check: StoreCheck<I, P>,
    inventory: I2,
    deployer: D,
    exe: E,
    store: VersionStore,
    journal: JournalStore,
    // the pause between the two tries to put the registered version back
    restore_retry_delay: Duration,
}

impl<I, P, I2, D, E> InstallService<I, P, I2, D, E>
where
    I: PackageInventory,
    P: ProcessInspector,
    I2: PackageInventory,
    D: PackageDeployer,
    E: ExeCopier,
{
    const APP_ID: &'static str = "Game";
    // files a registered game folder must hold to count as intact
    const REQUIRED_FILES: [&'static str; 2] = ["Minecraft.Windows.exe", "MicrosoftGame.Config"];

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        paths: AmmlPaths,
        catalogue: Catalogue,
        downloader: Downloader,
        cache: DownloadCache,
        check: StoreCheck<I, P>,
        inventory: I2,
        deployer: D,
        exe: E,
        restore_retry_delay: Duration,
    ) -> InstallService<I, P, I2, D, E> {
        InstallService {
            store: VersionStore::new(paths.clone()),
            journal: JournalStore::new(paths.journal()),
            paths,
            catalogue,
            downloader,
            cache,
            check,
            inventory,
            deployer,
            exe,
            restore_retry_delay,
        }
    }

    pub fn install(
        &self,
        request: &InstallRequest,
        on_step: &mut impl FnMut(InstallStep, Option<DownloadProgress>),
    ) -> Result<InstallOutcome, GdkError> {
        let _lock = InstallLock::acquire(&self.paths.lock())?;
        self.announce(on_step, InstallStep::Checking, &request.version);
        self.recover_locked()?;
        self.store.clean_stale()?;

        let Some(registered) = self.inventory.registered(request.channel.family_name())? else {
            return Err(self.missing_refusal(request.channel)?);
        };
        if self.store.get(&request.version).is_some() {
            return Err(GdkError::VersionExists(request.version.instance_name()));
        }
        let outcome = if registered.version == request.version {
            self.copy_out(request, &registered, on_step)?
        } else {
            self.install_and_restore(request, &registered, on_step)?
        };
        self.announce(on_step, InstallStep::Done, &request.version);
        Ok(outcome)
    }

    // Finishes an install that was stopped half way. Returns the Bedrock package registered afterwards when a
    // journal was found.
    pub fn recover(&self) -> Result<Option<String>, GdkError> {
        let _lock = InstallLock::acquire(&self.paths.lock())?;
        self.recover_locked()
    }

    fn recover_locked(&self) -> Result<Option<String>, GdkError> {
        let Some(journal) = self.journal.read()? else {
            return Ok(None);
        };
        curia::warn!("recovering an interrupted install", {
            original: journal.original_full_name.clone(),
            wanted: journal.wanted_full_name.clone(),
        });
        let family = journal.channel.family_name();
        if let Err(error) = self.remove_wanted(&journal, family, &[]) {
            curia::warn!("the wanted package was not removed", { error: error.to_string() });
        }
        let (final_name, _) = self.restore(&journal, family)?;
        self.clear_journal();
        self.remove_staging(&journal);
        Ok(Some(final_name))
    }

    fn clear_journal(&self) {
        if let Err(error) = self.journal.clear() {
            curia::warn!("the install journal was not deleted", { error: error.to_string() });
        }
    }

    // Deletes the staging folder a journal names, only when it is a staging folder of the version store. A
    // folder that cannot be deleted is left for `clean_stale`.
    fn remove_staging(&self, journal: &InstallJournal) {
        let Some(staging) = &journal.staging else { return };
        let is_staging = staging.parent().is_some_and(|parent| parent == self.paths.versions())
            && staging.file_name().is_some_and(|name| name.to_string_lossy().starts_with(VersionStore::STAGING_PREFIX));
        if !is_staging {
            curia::warn!("the journal names a folder that is not a staging folder", {
                path: staging.display().to_string(),
            });
            return;
        }
        if let Err(error) = TreeOps::remove(staging) {
            curia::warn!("a staging folder was not deleted", {
                path: staging.display().to_string(),
                error: error.to_string(),
            });
        }
    }

    fn copy_out(
        &self,
        request: &InstallRequest,
        registered: &RegisteredPackage,
        on_step: &mut impl FnMut(InstallStep, Option<DownloadProgress>),
    ) -> Result<InstallOutcome, GdkError> {
        let (_, bytes) = TreeOps::stats(&registered.content_location)?;
        let package = self.require_ready(request.channel, SpaceNeed::for_copy_out(bytes), false)?;
        let (expected, _) = TreeOps::stats(&package.install_location)?;
        let version = self.build_version(&package, &request.version, expected, None, on_step)?;
        Ok(InstallOutcome {
            version,
            path_used: "copy-out",
            final_full_name: package.full_name,
            store_updated_meanwhile: false,
        })
    }

    fn install_and_restore(
        &self,
        request: &InstallRequest,
        registered: &RegisteredPackage,
        on_step: &mut impl FnMut(InstallStep, Option<DownloadProgress>),
    ) -> Result<InstallOutcome, GdkError> {
        let family = request.channel.family_name();
        let current = self
            .catalogue
            .find_by_full_name(&registered.full_name)
            .ok_or_else(|| GdkError::CurrentNotInCatalogue(registered.full_name.clone()))?;
        let wanted = self
            .catalogue
            .find(&request.version, request.channel)
            .ok_or_else(|| GdkError::UnknownVersion(request.version.to_string()))?;
        let (Some(current_file), Some(wanted_file), Some(wanted_name)) =
            (current.file_name(), wanted.file_name(), wanted.package_full_name())
        else {
            return Err(GdkError::BadCatalogue("package urls"));
        };

        let current_size = self.downloader.remote_size(&current.urls)?;
        let wanted_size = self.downloader.remote_size(&wanted.urls)?;
        let current_cached = self.cache.cached(&current_file, current_size);
        let wanted_cached = self.cache.cached(&wanted_file, wanted_size);
        let need = SpaceNeed::for_install(wanted_size, current_size, wanted_cached.is_some(), current_cached.is_some());
        self.require_ready(request.channel, need, true)?;

        // the registered version's package comes first: the install below deletes its files
        let current_package = match current_cached {
            Some(path) => path,
            None => {
                self.announce(on_step, InstallStep::DownloadingCurrent, &request.version);
                self.downloader.fetch(&current.urls, &current_file, &mut |progress| {
                    on_step(InstallStep::DownloadingCurrent, Some(progress))
                })?
            }
        };
        let wanted_package = match wanted_cached {
            Some(path) => path,
            None => {
                self.announce(on_step, InstallStep::DownloadingWanted, &request.version);
                self.downloader.fetch(&wanted.urls, &wanted_file, &mut |progress| {
                    on_step(InstallStep::DownloadingWanted, Some(progress))
                })?
            }
        };

        // the downloads take time; nothing may have changed before the first registration change
        let again = self.require_ready(request.channel, SpaceNeed { work_drive: 0, game_drive: 0 }, true)?;
        if !again.full_name.eq_ignore_ascii_case(&registered.full_name) {
            return Err(GdkError::UnexpectedRegistration {
                expected: registered.full_name.clone(),
                found: again.full_name,
            });
        }

        let mut journal = InstallJournal {
            original_full_name: registered.full_name.clone(),
            original_package: current_package,
            wanted_full_name: wanted_name,
            staging: None,
            channel: request.channel,
        };
        self.journal.write(&journal)?;

        // the package that is registered now is never removed, whatever the journal says
        let started_with = [registered.full_name.as_str()];
        let work = self.change_and_extract(&mut journal, &wanted_package, request, on_step);

        self.announce(on_step, InstallStep::Removing, &request.version);
        if let Err(error) = self.remove_wanted(&journal, family, &started_with) {
            curia::warn!("the wanted package was not removed", { error: error.to_string() });
        }
        self.announce(on_step, InstallStep::Restoring, &request.version);
        let (final_full_name, store_updated_meanwhile) = self.restore(&journal, family)?;
        self.clear_journal();
        self.announce(on_step, InstallStep::Verifying, &request.version);

        let version = work?;
        let keep = self.cached_files_to_keep(request.channel, &current_file);
        let keep: Vec<&str> = keep.iter().map(String::as_str).collect();
        let _ = self.cache.retain_only(&keep);
        Ok(InstallOutcome { version, path_used: "install-restore", final_full_name, store_updated_meanwhile })
    }

    // the cached package of the registered version of each channel
    fn cached_files_to_keep(&self, channel: Channel, current_file: &str) -> Vec<String> {
        let mut keep = vec![current_file.to_string()];
        let other = match channel {
            Channel::Release => Channel::Preview,
            Channel::Preview => Channel::Release,
        };
        if let Ok(Some(package)) = self.inventory.registered(other.family_name())
            && let Some(file) = self.catalogue.find_by_full_name(&package.full_name).and_then(|entry| entry.file_name())
        {
            keep.push(file);
        }
        keep
    }

    fn change_and_extract(
        &self,
        journal: &mut InstallJournal,
        wanted_package: &Path,
        request: &InstallRequest,
        on_step: &mut impl FnMut(InstallStep, Option<DownloadProgress>),
    ) -> Result<VersionFolder, GdkError> {
        self.announce(on_step, InstallStep::Installing, &request.version);
        self.deployer.add(wanted_package)?;
        let found = self.inventory.registered(request.channel.family_name())?;
        // the wanted package is the one with the wanted version; its real full name may differ from the file name
        let registered = match found {
            Some(package)
                if package.version == request.version
                    && !package.full_name.eq_ignore_ascii_case(&journal.original_full_name) =>
            {
                package
            }
            other => {
                return Err(GdkError::UnexpectedRegistration {
                    expected: journal.wanted_full_name.clone(),
                    found: other.map(|package| package.full_name).unwrap_or_default(),
                });
            }
        };
        if !registered.full_name.eq_ignore_ascii_case(&journal.wanted_full_name) {
            journal.wanted_full_name = registered.full_name.clone();
            self.journal.write(journal)?;
        }
        let (expected, _) = TreeOps::stats(&registered.install_location)?;
        self.build_version(&registered, &request.version, expected, Some(journal), on_step)
    }

    // Copies a registered package into the version store. The staging folder is gone again on every failure.
    fn build_version(
        &self,
        package: &RegisteredPackage,
        version: &GameVersion,
        expected_files: u64,
        journal: Option<&InstallJournal>,
        on_step: &mut impl FnMut(InstallStep, Option<DownloadProgress>),
    ) -> Result<VersionFolder, GdkError> {
        let staging = self.store.begin()?;
        let temp = std::env::temp_dir().join(format!("amml-exe-{}", uuid::Uuid::new_v4().simple()));
        let result = self.fill(&staging, &temp, package, version, expected_files, journal, on_step);
        if let Err(error) = TreeOps::remove(&temp) {
            curia::warn!("a temporary folder was left behind", {
                path: temp.display().to_string(),
                error: error.to_string(),
            });
        }
        if result.is_err() {
            let _ = TreeOps::remove(&staging);
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn fill(
        &self,
        staging: &Path,
        temp: &Path,
        package: &RegisteredPackage,
        version: &GameVersion,
        expected_files: u64,
        journal: Option<&InstallJournal>,
        on_step: &mut impl FnMut(InstallStep, Option<DownloadProgress>),
    ) -> Result<VersionFolder, GdkError> {
        if let Some(journal) = journal {
            self.journal.write(&InstallJournal { staging: Some(staging.to_path_buf()), ..journal.clone() })?;
        }
        self.announce(on_step, InstallStep::CopyingExe, version);
        let exe = self.exe.copy_exe(package, Self::APP_ID, temp)?;
        self.announce(on_step, InstallStep::CopyingFiles, version);
        self.store.copy_tree(&package.install_location, staging, true)?;
        self.store.commit(staging, version, exe, expected_files)
    }

    // Is this registered package the wanted one? It has the wanted name, or the wanted version under another name.
    fn is_wanted(journal: &InstallJournal, package: &RegisteredPackage) -> bool {
        if package.full_name.eq_ignore_ascii_case(&journal.original_full_name) {
            return false;
        }
        package.full_name.eq_ignore_ascii_case(&journal.wanted_full_name)
            || Self::package_version(&journal.wanted_full_name).is_some_and(|wanted| wanted == package.version)
    }

    // Removes the wanted package when it is registered. The original, and every name in `protected`, is refused
    // here and again by the deployer.
    fn remove_wanted(&self, journal: &InstallJournal, family: &str, protected: &[&str]) -> Result<(), GdkError> {
        let Some(now) = self.inventory.registered(family)? else {
            return Ok(());
        };
        if !Self::is_wanted(journal, &now) {
            return Ok(());
        }
        let mut never: Vec<String> = vec![journal.original_full_name.clone()];
        never.extend(protected.iter().map(|name| name.to_string()));
        if never.iter().any(|name| name.eq_ignore_ascii_case(&now.full_name)) {
            return Err(GdkError::Protected(now.full_name));
        }
        self.deployer.remove(&now.full_name, &never)
    }

    fn files_present(package: &RegisteredPackage) -> bool {
        Self::REQUIRED_FILES.iter().all(|name| package.install_location.join(name).is_file())
    }

    // Puts the original package back. A newer healthy version that the Store installed meanwhile is accepted.
    // Returns the registered full name and whether the Store moved on. Every failure is a `RestoreFailed`.
    fn restore(&self, journal: &InstallJournal, family: &str) -> Result<(String, bool), GdkError> {
        let mut cause = String::new();
        match self.already_restored(journal, family) {
            Ok(Some(done)) => return Ok(done),
            Ok(None) => {}
            Err(error) => cause = error.to_string(),
        }
        for attempt in 0..2 {
            match self.add_original(journal, family) {
                Ok(done) => return Ok(done),
                Err(text) => cause = text,
            }
            if attempt == 0 {
                std::thread::sleep(self.restore_retry_delay);
            }
        }
        curia::error!("the registered version was not restored", {
            original: journal.original_full_name.clone(),
            cause: cause.clone(),
        });
        Err(GdkError::RestoreFailed {
            original: journal.original_full_name.clone(),
            package: journal.original_package.clone(),
            cause,
        })
    }

    // The original, or a newer version from the Store, is registered, healthy, and has its files.
    fn already_restored(&self, journal: &InstallJournal, family: &str) -> Result<Option<(String, bool)>, GdkError> {
        let Some(now) = self.inventory.registered(family)? else {
            return Ok(None);
        };
        if !now.status_ok || !Self::files_present(&now) {
            return Ok(None);
        }
        if now.full_name.eq_ignore_ascii_case(&journal.original_full_name) {
            return Ok(Some((now.full_name, false)));
        }
        let newer = Self::package_version(&journal.original_full_name).is_some_and(|original| now.version > original);
        if newer && !Self::is_wanted(journal, &now) {
            return Ok(Some((now.full_name, true)));
        }
        Ok(None)
    }

    fn add_original(&self, journal: &InstallJournal, family: &str) -> Result<(String, bool), String> {
        self.deployer.add(&journal.original_package).map_err(|error| error.to_string())?;
        match self.inventory.registered(family).map_err(|error| error.to_string())? {
            Some(now)
                if now.full_name.eq_ignore_ascii_case(&journal.original_full_name)
                    && now.status_ok
                    && Self::files_present(&now) =>
            {
                Ok((now.full_name, false))
            }
            Some(now) => Err(format!("{} is registered after the restore, and is not intact", now.full_name)),
            None => Err("nothing is registered after the restore".to_string()),
        }
    }

    fn package_version(full_name: &str) -> Option<GameVersion> {
        GameVersion::from_package_version(full_name.split('_').nth(1)?).ok()
    }

    fn require_ready(
        &self,
        channel: Channel,
        need: SpaceNeed,
        block_running: bool,
    ) -> Result<RegisteredPackage, GdkError> {
        match self.check.check(channel, need, &self.paths.root(), block_running)? {
            StoreState::Ready(package) => Ok(package),
            other => Err(GdkError::NotReady(Box::new(other))),
        }
    }

    // the refusal to give when the channel has no registered package
    fn missing_refusal(&self, channel: Channel) -> Result<GdkError, GdkError> {
        let state = self.check.check(channel, SpaceNeed { work_drive: 0, game_drive: 0 }, &self.paths.root(), false)?;
        Ok(GdkError::NotReady(Box::new(match state {
            StoreState::Ready(_) => match channel {
                Channel::Release => StoreState::BedrockMissing,
                Channel::Preview => StoreState::PreviewMissing,
            },
            other => other,
        })))
    }

    fn announce(
        &self,
        on_step: &mut impl FnMut(InstallStep, Option<DownloadProgress>),
        step: InstallStep,
        version: &GameVersion,
    ) {
        curia::info!("install step", { step: format!("{step:?}"), version: version.to_string() });
        on_step(step, None);
    }
}

impl<I, P, I2, D, E> Installer for InstallService<I, P, I2, D, E>
where
    I: PackageInventory,
    P: ProcessInspector,
    I2: PackageInventory,
    D: PackageDeployer,
    E: ExeCopier,
{
    fn install(
        &self,
        request: &InstallRequest,
        on_step: &mut dyn FnMut(InstallStep, Option<DownloadProgress>),
    ) -> Result<InstallOutcome, GdkError> {
        InstallService::install(self, request, &mut |step, progress| on_step(step, progress))
    }

    fn recover(&self) -> Result<Option<String>, GdkError> {
        InstallService::recover(self)
    }
}
