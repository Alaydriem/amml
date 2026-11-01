use crate::{Library, ModExports, ModPhase};

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
    /// Mod folder, config folder and data folder. The boxed strings never move, so `mod_dir` can
    /// hand out views of them.
    pub dirs: Box<[String; 3]>,
}
