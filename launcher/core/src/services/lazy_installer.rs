use amml_gdk::{
    AmmlPaths, CancelFlag, Catalogue, CatalogueReader, DiskSpace, DownloadCache, DownloadProgress, Downloader,
    GdkError, InstallOutcome, InstallRequest, InstallService, InstallStep, Installer, StoreCheck, WindowsExeCopier,
    WindowsPackageDeployer, WindowsPackageInventory, WindowsProcessInspector,
};
use std::time::Duration;

// The Store install service, built when it is first needed so that commands which never install a game version
// do not fetch the version catalogue.
pub struct LazyInstaller {
    paths: AmmlPaths,
    cancel: CancelFlag,
}

type StoreInstall = InstallService<
    WindowsPackageInventory,
    WindowsProcessInspector,
    WindowsPackageInventory,
    WindowsPackageDeployer,
    WindowsExeCopier,
>;

impl LazyInstaller {
    pub fn new(paths: AmmlPaths, cancel: CancelFlag) -> LazyInstaller {
        LazyInstaller { paths, cancel }
    }

    fn agent() -> ureq::Agent {
        ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(30)))
            .timeout_recv_response(Some(Duration::from_secs(60)))
            .timeout_per_read(Some(Duration::from_secs(60)))
            .build()
            .into()
    }

    fn service(&self, agent: ureq::Agent, catalogue: Catalogue) -> StoreInstall {
        InstallService::new(
            self.paths.clone(),
            catalogue,
            Downloader::new(agent, self.paths.downloads(), Downloader::DEFAULT_ATTEMPTS, self.cancel.clone()),
            DownloadCache::new(self.paths.downloads()),
            StoreCheck::new(WindowsPackageInventory::new(), WindowsProcessInspector::new(), DiskSpace::new()),
            WindowsPackageInventory::new(),
            WindowsPackageDeployer::new(),
            WindowsExeCopier::new(WindowsExeCopier::DEFAULT_TIMEOUT),
            Duration::from_secs(10),
        )
    }
}

impl Installer for LazyInstaller {
    fn install(
        &self,
        request: &InstallRequest,
        on_step: &mut dyn FnMut(InstallStep, Option<DownloadProgress>),
    ) -> Result<InstallOutcome, GdkError> {
        let agent = Self::agent();
        let catalogue = CatalogueReader::fetch(&agent)?;
        Installer::install(&self.service(agent, catalogue), request, on_step)
    }

    fn recover(&self) -> Result<Option<String>, GdkError> {
        let service = self.service(Self::agent(), Catalogue::new(Vec::new(), Vec::new()));
        Installer::recover(&service)
    }
}
