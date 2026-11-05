use super::{FakeInstaller, FakePatcher, FakeProcesses, FakeServer, RegistryTree, Route, Zips};
use amml_gdk::{Downloader, InstanceService, VersionStore};
use amml_launcher_core::{
    AmmlInstaller, AssetFetcher, CacheLock, InstanceGuard, InstanceManager, LauncherPaths, ModCatalogue, ModInstaller,
    RegistryCache, RegistryClient, RegistryRef, RegistrySet, UreqHttpFetch,
};
use amml_registry::{AmmlRelease, HashText, ModDocument};
use std::path::PathBuf;
use std::sync::Arc;

// A launcher root in a temp folder, one fake server that is the official registry and the host of every zip,
// and fakes for the Store install, the patcher and the process list.
pub struct Harness {
    pub root: tempfile::TempDir,
    pub paths: LauncherPaths,
    pub server: FakeServer,
    pub installer: FakeInstaller,
    pub processes: FakeProcesses,
    pub patcher: FakePatcher,
    work: tempfile::TempDir,
}

impl Harness {
    pub fn new() -> Harness {
        let root = tempfile::tempdir().unwrap();
        let paths = super::Roots::paths(root.path());
        let installer = FakeInstaller::new(paths.gdk().clone());
        Harness {
            paths,
            root,
            server: FakeServer::start(vec![]),
            installer,
            processes: FakeProcesses::new(),
            patcher: FakePatcher::new(),
            work: tempfile::tempdir().unwrap(),
        }
    }

    pub fn official(&self) -> RegistryRef {
        RegistryRef::official_at(self.server.base())
    }

    // serves an AMML zip and describes it
    pub fn amml_release(&self, version: &str, api: u32, games: &[&str]) -> AmmlRelease {
        let zip = Zips::amml_zip(&self.work.path().join(format!("amml-{version}.zip")), version, true);
        let bytes = std::fs::read(zip).unwrap();
        let path = format!("/zips/amml-{version}.zip");
        self.server.replace(&path, Route::new(bytes.clone()));
        let mut release = RegistryTree::amml_release(version, api, games);
        release.asset = self.server.url(&path);
        release.sha256 = HashText::sha256(&bytes);
        release.size = bytes.len() as u64;
        release
    }

    // serves a mod zip and returns a document that lists it
    pub fn mod_document(
        &self,
        namespace: &str,
        name: &str,
        core: bool,
        default: bool,
        version: &str,
        api: u32,
    ) -> ModDocument {
        let release = super::Catalogues::serve_mod(&self.server, self.work.path(), namespace, name, version, api, &[]);
        RegistryTree::document(namespace, name, core, default, vec![release])
    }

    pub fn publish(&self, amml: Vec<AmmlRelease>, documents: &[ModDocument]) {
        RegistryTree::publish(&self.server, "Official", &RegistryTree::amml(amml), documents);
    }

    // the bytes of a served zip are replaced by other bytes of the same length
    pub fn corrupt_zip(&self, path: &str) {
        let mut bytes = std::fs::read(self.work.path().join(path)).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
        self.server.replace(&format!("/zips/{path}"), Route::new(bytes));
    }

    pub fn guard(&self) -> InstanceGuard<FakeProcesses> {
        InstanceGuard::new(Arc::new(self.processes.clone()))
    }

    pub fn catalogue(&self) -> ModCatalogue<UreqHttpFetch> {
        let client = RegistryClient::new(
            self.official(),
            UreqHttpFetch::new(UreqHttpFetch::agent()),
            RegistryCache::new(self.paths.registry_cache(), RegistryCache::BUDGET, CacheLock::new()),
        );
        let downloads: PathBuf = self.paths.gdk().downloads();
        let downloader = Downloader::new(UreqHttpFetch::agent(), downloads.clone(), 1, amml_gdk::CancelFlag::new());
        ModCatalogue::new(RegistrySet::new(vec![client]), AssetFetcher::new(downloader, downloads))
    }

    pub fn manager(&self) -> InstanceManager<FakeInstaller, FakePatcher, FakeProcesses, UreqHttpFetch> {
        self.manager_with(self.catalogue())
    }

    pub fn manager_with(
        &self,
        catalogue: ModCatalogue<UreqHttpFetch>,
    ) -> InstanceManager<FakeInstaller, FakePatcher, FakeProcesses, UreqHttpFetch> {
        let gdk = self.paths.gdk().clone();
        InstanceManager::new(
            self.paths.clone(),
            self.installer.clone(),
            InstanceService::new(gdk.clone(), VersionStore::new(gdk.clone())),
            VersionStore::new(gdk),
            catalogue,
            AmmlInstaller::new(self.patcher.clone(), self.guard()),
            ModInstaller::new(self.guard()),
            self.guard(),
        )
    }

    pub fn downloads_empty(&self) -> bool {
        match std::fs::read_dir(self.paths.gdk().downloads()) {
            Ok(entries) => entries.count() == 0,
            Err(_) => true,
        }
    }
}
