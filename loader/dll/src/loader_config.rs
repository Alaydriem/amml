use amml_runtime::EntryTable;

use crate::LoaderDirs;

/// What a host gives `LoaderState::init`.
pub struct LoaderConfig {
    /// Base address of the game module; `function` and `global` entries are added to it.
    pub module_base: u64,
    /// The entries that resolved, for every mod and for the loader itself.
    pub table: EntryTable,
    pub dirs: LoaderDirs,
    /// Most hooks, and most distinct hooked functions, for the life of the process.
    pub max_hooks: usize,
}

impl LoaderConfig {
    pub const DEFAULT_MAX_HOOKS: usize = 256;

    pub fn new(module_base: u64, table: EntryTable, dirs: LoaderDirs) -> Self {
        Self { module_base, table, dirs, max_hooks: Self::DEFAULT_MAX_HOOKS }
    }
}
