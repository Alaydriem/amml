use super::{FakeInstaller, FakePatcher, FakeProcesses};
use amml_gdk::{Downloader, GameVersion, InstanceService, VersionStore};
use amml_launcher_core::{
    AmmlInstaller, AssetFetcher, CacheLock, DataChoice, InstanceGuard, InstanceManager, InstanceReader, LauncherError,
    LauncherPaths, ModCatalogue, ModChange, ModId, ModInstaller, ModSource, NewInstanceOutcome, NewInstanceRequest,
    RegistryCache, RegistryClient, RegistryRef, RegistrySet, UpdateChecker, UpdateRow, UpdateStatus, UreqHttpFetch,
    UrlInspector,
};
use amml_proof_fixture::ProofTree;
use std::path::PathBuf;
use std::sync::Arc;

pub type ProofManager = InstanceManager<FakeInstaller, FakePatcher, FakeProcesses, UreqHttpFetch>;

// A launcher root in a temp folder whose official registry is the tree the real registry tool built, served over
// local HTTP. Only the Store install, the patcher and the process list are fakes.
pub struct ProofHarness {
    pub root: tempfile::TempDir,
    pub paths: LauncherPaths,
    pub proof: ProofTree,
    pub installer: FakeInstaller,
    pub processes: FakeProcesses,
    pub patcher: FakePatcher,
    // requests the server had already answered when the tree was built (the registry tool's own downloads)
    pub built_requests: usize,
    _work: tempfile::TempDir,
}

impl ProofHarness {
    pub fn new() -> ProofHarness {
        Self::block_foreign_hosts();
        let work = tempfile::tempdir().unwrap();
        let proof = ProofTree::build(work.path());
        let built_requests = proof.server.requests().len();
        let root = tempfile::tempdir().unwrap();
        let paths = super::Roots::paths(root.path());
        let installer = FakeInstaller::new(paths.gdk().clone());
        ProofHarness {
            paths,
            root,
            proof,
            installer,
            processes: FakeProcesses::new(),
            patcher: FakePatcher::new(),
            built_requests,
            _work: work,
        }
    }

    // ureq 3.5 reads these variables when an agent is built (Proxy::try_from_env). Every agent of a proof test is
    // built after this call, so a request to any host but 127.0.0.1 / localhost goes to a dead proxy port and fails.
    // nextest runs each test in its own process, and no thread has started before the first call.
    fn block_foreign_hosts() {
        for name in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"] {
            // SAFETY: the proof harness runs this first in the test, before it starts a thread of its own.
            unsafe { std::env::set_var(name, "http://127.0.0.1:9") };
        }
        for name in ["NO_PROXY", "no_proxy"] {
            // SAFETY: as above.
            unsafe { std::env::set_var(name, "127.0.0.1,localhost") };
        }
    }

    // the request paths made after the tree was built, so by the launcher only
    pub fn launcher_requests(&self) -> Vec<String> {
        self.proof.server.requests().split_off(self.built_requests)
    }

    pub fn guard(&self) -> InstanceGuard<FakeProcesses> {
        InstanceGuard::new(Arc::new(self.processes.clone()))
    }

    pub fn manager(&self) -> ProofManager {
        let client = RegistryClient::new(
            RegistryRef::official_at(format!("{}/", self.proof.registry_base)),
            UreqHttpFetch::new(UreqHttpFetch::agent()),
            RegistryCache::new(self.paths.registry_cache(), RegistryCache::BUDGET, CacheLock::new()),
        );
        let downloads: PathBuf = self.paths.gdk().downloads();
        let downloader = Downloader::new(UreqHttpFetch::agent(), downloads.clone(), 1, amml_gdk::CancelFlag::new());
        let catalogue = ModCatalogue::new(RegistrySet::new(vec![client]), AssetFetcher::new(downloader, downloads));
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

    // makes an instance through the real create: AMML and the default mods come from the tree
    pub fn create(&self, game: &str, name: &str) -> Result<NewInstanceOutcome, LauncherError> {
        let request = NewInstanceRequest {
            game: GameVersion::parse(game).unwrap(),
            preview: false,
            name: name.to_string(),
            amml: None,
            data: DataChoice::Shared(self.paths.default_data_dir()),
        };
        self.manager().create(&request, &mut |_| {})
    }

    // an instance for `game`, named after it, with AMML and the default mods
    pub fn instance_on(&self, game: &str) -> String {
        let name = format!("g{}", game.replace('.', "-"));
        self.create(game, &name).unwrap();
        name
    }

    // removes every mod, so a test can install one at a time
    pub fn clear_mods(&self, name: &str) {
        let state = InstanceReader::read(&self.paths.instance(name)).unwrap();
        let mut manager = self.manager();
        for installed in state.mods {
            manager.remove_mod(name, &installed.id).unwrap();
        }
    }

    // installs a release by the URL of its zip: the pinned install path (the real inspector and installer)
    pub fn pin(&self, name: &str, mod_name: &str, version: &str) -> ModChange {
        let zip = self.proof.mod_zip(mod_name, version);
        let relative = zip.strip_prefix(self.proof.tree.parent().unwrap()).unwrap();
        let parts: Vec<String> =
            relative.components().map(|part| part.as_os_str().to_string_lossy().to_string()).collect();
        let url = format!("{}/{}", self.proof.server.base(), parts.join("/"));
        let mut manager = self.manager();
        let inspection = UrlInspector::inspect(manager.catalogue_mut(), &url, &mut |_| {}).unwrap();
        assert!(inspection.matched.is_some(), "the pinned zip of {mod_name} {version} matches a registry release");
        manager.install_inspected(name, inspection, false).unwrap()
    }

    // installs the newest release that fits, as `amml-launcher mod add <id>` does
    pub fn install(&self, name: &str, mod_name: &str) -> Result<ModChange, LauncherError> {
        let id = ModId::parse(&format!("@proof/{mod_name}")).unwrap();
        let mut manager = self.manager();
        let resolved = manager.resolve_mod(name, &id, None)?;
        manager.install_mod(name, &resolved, &mut |_| {})
    }

    // resolves with the recorded source, as an update does, then installs it; returns the resolved version
    pub fn resolve_and_install(&self, name: &str, mod_name: &str) -> (String, ModChange) {
        let id = ModId::parse(&format!("@proof/{mod_name}")).unwrap();
        let held = InstanceReader::read(&self.paths.instance(name)).unwrap();
        let source = held.mods.iter().find(|installed| installed.id == id).map(|installed| installed.source.clone());
        let mut manager = self.manager();
        let resolved = manager.resolve_mod(name, &id, source.as_ref()).unwrap();
        let version = resolved.release.version.clone();
        (version, manager.install_mod(name, &resolved, &mut |_| {}).unwrap())
    }

    pub fn check(&self, name: &str) -> Vec<UpdateRow> {
        self.manager().check_updates(name).unwrap()
    }

    // the same path as `amml-launcher mod update --all`: every Available row is resolved against the registry the mod
    // came from and installed
    pub fn update_all(&self, name: &str) -> Vec<(String, ModChange)> {
        let mut manager = self.manager();
        let rows = manager.check_updates(name).unwrap();
        let held = InstanceReader::read(&self.paths.instance(name)).unwrap();
        let mut changes = Vec::new();
        for row in rows.iter().filter(|row| row.id != UpdateChecker::AMML_ID && row.status == UpdateStatus::Available) {
            let id = ModId::parse(&row.id).unwrap();
            let source: Option<ModSource> =
                held.mods.iter().find(|installed| installed.id == id).map(|installed| installed.source.clone());
            let resolved = manager.resolve_mod(name, &id, source.as_ref()).unwrap();
            changes.push((row.id.clone(), manager.install_mod(name, &resolved, &mut |_| {}).unwrap()));
        }
        changes
    }

    // the `version` the installed `amml.hcl` of a mod holds, or None when the mod folder is missing
    pub fn version_on_disk(&self, name: &str, mod_name: &str) -> Option<String> {
        let id = ModId::parse(&format!("@proof/{mod_name}")).unwrap();
        let folder = id.folder(&InstanceReader::mods_dir(&self.paths.instance(name), true));
        let text = std::fs::read_to_string(folder.join("amml.hcl")).ok()?;
        text.lines().find_map(|line| {
            let rest = line.trim().strip_prefix("version")?.trim_start().strip_prefix("=")?;
            Some(rest.trim().trim_matches('"').to_string())
        })
    }

    pub fn folder_exists(&self, name: &str, mod_name: &str) -> bool {
        let id = ModId::parse(&format!("@proof/{mod_name}")).unwrap();
        id.folder(&InstanceReader::mods_dir(&self.paths.instance(name), true)).exists()
    }
}
