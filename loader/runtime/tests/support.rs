use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use amml_data::{
    BuildData, BuildDataStore, BuildId, BuildIdReader, Entry, EntryId, EntryKind, OperandKind, Recipe, RecipeLoader,
    RecipeSet, Resolved, Step,
};
use amml_runtime::{
    CacheFile, CacheSource, DeriveSource, HostResolver, ImageLayout, LeviLaminaSource, RecipeDigest, SymbolLookup,
};
use pelite::pe64::exports::GetProcAddress;
use pelite::pe64::{Pe, PeFile};

pub const BASE: u64 = 0x1_4000_0000;
// image size of `Fixtures::layout`
pub const IMAGE_SIZE: u64 = 0x100_0000;
// start of the non-executable tail of `Fixtures::layout`
pub const DATA_START: u64 = 0xF0_0000;

pub struct Fixtures;

// The recipes and data directories of a run on the probe dll.
pub struct ProbeDir {
    pub recipes: PathBuf,
    pub data: PathBuf,
    // RVA of the exported `amml_probe`
    pub rva: u32,
}

impl Fixtures {
    // an image whose code is 0x100..DATA_START and whose tail up to IMAGE_SIZE is data
    pub fn layout() -> ImageLayout {
        ImageLayout::new(IMAGE_SIZE, std::iter::once(0x100..DATA_START).collect())
    }

    pub fn id(name: &str) -> EntryId {
        EntryId(name.to_string())
    }

    // a valid recipe for `kind`; the steps are never run by the sources under test
    pub fn entry(name: &str, kind: EntryKind, msvc_name: Option<&str>) -> Entry {
        let pattern = Step::Pattern { bytes: "AA BB".to_string(), offset: 0, literal_ok: Vec::new() };
        let steps = match kind {
            EntryKind::Function | EntryKind::Global => vec![pattern],
            EntryKind::VSlot => vec![pattern, Step::SlotIn { class: ".?AVX@@".to_string() }],
            EntryKind::Field => vec![pattern, Step::Operand { index: 0, kind: OperandKind::Displacement }],
        };
        Entry {
            id: Self::id(name),
            kind,
            msvc_name: msvc_name.map(str::to_string),
            recipes: vec![Recipe { steps }],
            checks: Vec::new(),
            notes: String::new(),
            hooked: false,
        }
    }

    pub fn set(entries: Vec<Entry>) -> RecipeSet {
        RecipeSet::new(entries).unwrap()
    }

    pub fn resolved(kind: EntryKind, value: u64) -> Resolved {
        Resolved { kind, value, recipes_agreeing: 2 }
    }

    // writes the cache file of (`file_for`, `digest`) holding `contents_build` and `entries`
    pub fn write_cache(
        dir: &Path,
        file_for: BuildId,
        digest: &str,
        contents_build: BuildId,
        entries: Vec<(&str, Resolved)>,
    ) {
        let mut data = BuildData::new(contents_build);
        for (name, resolved) in entries {
            data.entries.insert(Self::id(name), resolved);
        }
        BuildDataStore::write(&CacheFile::path(dir, file_for, digest), &data).unwrap();
    }

    pub fn probe_dll() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../derive/cli/tests/fixtures/pe/small.dll")
    }

    // Recipes: Probe.run (function) and Probe.global (global) both match the prologue of `amml_probe`;
    // Probe.absent matches nothing.
    pub fn probe_dir(root: &Path) -> ProbeDir {
        let bytes = std::fs::read(Self::probe_dll()).unwrap();
        let file = PeFile::from_bytes(&bytes).unwrap();
        let rva = file.get_export("amml_probe").unwrap().symbol().unwrap();
        let prologue = file.derva_slice::<u8>(rva, 24).unwrap();
        let prologue = prologue.iter().map(|byte| format!("{byte:02X}")).collect::<Vec<_>>().join(" ");
        let entry = |name: &str, kind: &str, msvc: &str, bytes: &str, check: &str| {
            format!(
                "entry \"{name}\" {{\n  kind = \"{kind}\"\n  msvc_name = \"{msvc}\"\n  recipe {{\n    step \"pattern\" {{\n      bytes = \"{bytes}\"\n    }}\n  }}\n{check}}}\n\n"
            )
        };
        let check = "  check \"function_start\" {}\n";
        let text = entry("Probe.run", "function", "probe_run", &prologue, check)
            + &entry("Probe.global", "global", "probe_global", &prologue, "")
            + &entry("Probe.absent", "function", "probe_absent", "DE AD BE EF 01 02 03 04 05 06 07", "");
        let recipes = root.join("recipes");
        std::fs::create_dir_all(&recipes).unwrap();
        std::fs::write(recipes.join("probe.hcl"), text).unwrap();
        ProbeDir { recipes, data: root.join("data"), rva }
    }

    pub fn probe_build() -> BuildId {
        BuildIdReader::from_path(&Self::probe_dll()).unwrap()
    }

    pub fn probe_cache(probe: &ProbeDir) -> CacheSource {
        let set = RecipeLoader::load_dir(&probe.recipes).unwrap();
        let digest = RecipeDigest::of_dir(&probe.recipes).unwrap();
        CacheSource::new(probe.data.clone(), Self::probe_build(), digest, set)
    }

    pub fn probe_derive(probe: &ProbeDir) -> (DeriveSource, Arc<AtomicUsize>) {
        let derive = DeriveSource::new(Self::probe_dll(), probe.recipes.clone(), probe.data.clone());
        let runs = derive.run_counter();
        (derive, runs)
    }

    pub fn probe_standalone(probe: &ProbeDir) -> (HostResolver, Arc<AtomicUsize>) {
        let set = RecipeLoader::load_dir(&probe.recipes).unwrap();
        let (derive, runs) = Self::probe_derive(probe);
        (HostResolver::standalone(Self::probe_cache(probe), derive, set), runs)
    }

    pub fn probe_levilamina(probe: &ProbeDir, lookup: SymbolLookup) -> (HostResolver, Arc<AtomicUsize>) {
        let set = RecipeLoader::load_dir(&probe.recipes).unwrap();
        let (derive, runs) = Self::probe_derive(probe);
        let levilamina = LeviLaminaSource::new(lookup, set.clone(), BASE, Self::layout());
        (HostResolver::levilamina(Self::probe_cache(probe), derive, levilamina, set), runs)
    }
}
