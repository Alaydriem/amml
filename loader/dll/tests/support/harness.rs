use std::fs;
use std::path::{Path, PathBuf};

use amml::{LoaderConfig, LoaderDirs, LoaderState};
use amml_data::{EntryId, EntryKind, Resolved};
use amml_runtime::EntryTable;

use tempfile::{TempDir, tempdir};

use super::{HeapProbe, HookTarget};

/// A loader state over a temporary folder, with a small fixed entry table.
///
/// The loader state is process-wide, so a test that uses it runs in its own process (nextest).
pub struct Harness {
    pub root: TempDir,
    pub state: &'static LoaderState,
}

impl Harness {
    pub const BASE: u64 = 0x1000;

    pub fn start() -> Self {
        Self::start_with(true)
    }

    // `with_mod_data` false gives the loader no shared data root.
    pub fn start_with(with_mod_data: bool) -> Self {
        let root = tempdir().unwrap();
        let mod_data = with_mod_data.then(|| root.path().join("mod-data"));
        let dirs = LoaderDirs::new(root.path().join("mods"), root.path().join("logs"), mod_data);
        // SAFETY: the only entries that are called are the two heap entries, which point to `HeapProbe`.
        let state = unsafe { LoaderState::init(LoaderConfig::new(Self::BASE, Self::table(), dirs)) }.unwrap();
        Self { root, state }
    }

    pub fn logs(&self) -> PathBuf {
        self.root.path().join("logs")
    }

    pub fn mod_data(&self) -> PathBuf {
        self.root.path().join("mod-data")
    }

    pub fn mods(&self) -> PathBuf {
        self.root.path().join("mods")
    }

    // Writes `<mods>/@<namespace>/<name>/amml.hcl` and copies `dll` to `<mods>/@<namespace>/<name>/<dll_name>`.
    pub fn install_mod(&self, namespace: &str, name: &str, dll: &Path, dll_name: &str, api_version: u32) {
        self.install_mod_with(namespace, name, dll, dll_name, api_version, "");
    }

    // Like `install_mod`, with `amml_version = "<amml_version>"` in the manifest. The value is written as given.
    pub fn install_mod_needing(
        &self,
        namespace: &str,
        name: &str,
        dll: &Path,
        dll_name: &str,
        api_version: u32,
        amml_version: &str,
    ) {
        let line = format!("amml_version = \"{amml_version}\"\n");
        self.install_mod_with(namespace, name, dll, dll_name, api_version, &line);
    }

    // Like `install_mod`, with `extra` (whole HCL lines) appended to the manifest.
    pub fn install_mod_with(
        &self,
        namespace: &str,
        name: &str,
        dll: &Path,
        dll_name: &str,
        api_version: u32,
        extra: &str,
    ) {
        let dir = self.mods().join(format!("@{namespace}")).join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::copy(dll, dir.join(dll_name)).unwrap();
        let manifest = format!(
            "namespace = \"{namespace}\"\nname = \"{name}\"\nentry = \"{dll_name}\"\napi_version = {api_version}\nentries = [\"Echo.found\", \"Echo.absent\"]\n{extra}"
        );
        fs::write(dir.join("amml.hcl"), manifest).unwrap();
    }

    fn table() -> EntryTable {
        let ids = [
            "Echo.found",
            "Echo.absent",
            "Echo.slot",
            "Echo.offset",
            "Echo.global",
            "Echo.target",
            "core.operator_new",
            "core.operator_delete",
        ];
        let wanted: Vec<EntryId> = ids.iter().map(|id| EntryId((*id).to_string())).collect();
        let mut table = EntryTable::new(&wanted);
        let resolved = |kind, value| Resolved { kind, value, recipes_agreeing: 2 };
        let entries = [
            ("Echo.found", resolved(EntryKind::Function, 0x1234)),
            ("Echo.slot", resolved(EntryKind::VSlot, 12)),
            ("Echo.offset", resolved(EntryKind::Field, 0x30)),
            ("Echo.global", resolved(EntryKind::Global, 0x5000)),
            ("Echo.target", resolved(EntryKind::Function, HookTarget::address() - Self::BASE)),
            (
                "core.operator_new",
                resolved(EntryKind::Function, HeapProbe::alloc as *const () as usize as u64 - Self::BASE),
            ),
            (
                "core.operator_delete",
                resolved(EntryKind::Function, HeapProbe::free as *const () as usize as u64 - Self::BASE),
            ),
        ];
        for (id, value) in entries {
            table.insert(EntryId(id.to_string()), value, "cache");
        }
        table
    }
}
