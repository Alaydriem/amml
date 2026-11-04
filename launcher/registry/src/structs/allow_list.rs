use super::AllowedMod;

/// The policy file of a registry: which repos may publish, and which mods are core or default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowList {
    pub amml_repo: String,
    pub mods: Vec<AllowedMod>,
}

impl AllowList {
    pub fn find(&self, namespace: &str, name: &str) -> Option<&AllowedMod> {
        self.mods.iter().find(|entry| entry.namespace == namespace && entry.name == name)
    }
}
