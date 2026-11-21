use crate::{Library, ModExports, ModPhase, UseRow};

/// A mod the loader holds. Records are only added, never removed, and the library is never freed.
pub struct ModRecord {
    /// The mod id, `@<namespace>/<name>`.
    pub id: String,
    /// Kept to document that the mapping outlives every pointer a mod holds.
    pub _library: Library,
    pub exports: ModExports,
    pub phase: ModPhase,
    /// Handles of the hooks this mod installed.
    pub hooks: Vec<u64>,
    /// Mod folder, config folder, data folder and shared data folder. The shared data folder is empty
    /// when the host gave no root. The boxed strings never move, so `mod_dir` can hand out views of them.
    pub dirs: Box<[String; 4]>,
    /// The `uses` block of a v3 mod; empty for a v2 mod.
    pub uses: Vec<UseRow>,
    /// Events of required listens that were not available during the running load or enable call.
    pub missing_required: Vec<String>,
}
