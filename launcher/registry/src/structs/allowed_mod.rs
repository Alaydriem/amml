/// One `mod` block of the allow list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedMod {
    pub namespace: String,
    pub name: String,
    pub repo: String,
    // a core mod is also a default mod
    pub core: bool,
    pub default: bool,
}

impl AllowedMod {
    pub fn id(&self) -> String {
        format!("@{}/{}", self.namespace, self.name)
    }
}
