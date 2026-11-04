use crate::ResolvedMod;

// The mods a new instance gets: the core mods of the official registry and every `default` mod.
#[derive(Debug, Clone, Default)]
pub struct DefaultMods {
    pub mods: Vec<ResolvedMod>,
    // default mods of a registry that could not be read: id and reason
    pub skipped: Vec<(String, String)>,
}
