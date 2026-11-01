/// Which mod callback the loader is running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Callback {
    Load,
    Enable,
    Disable,
    Unload,
}

impl Callback {
    /// The name of the mod export this callback calls.
    pub fn export_name(self) -> &'static str {
        match self {
            Self::Load => "amml_mod_load",
            Self::Enable => "amml_mod_enable",
            Self::Disable => "amml_mod_disable",
            Self::Unload => "amml_mod_unload",
        }
    }

    /// Whether a mod may install a hook while this callback runs: only in load and enable.
    pub fn allows_hooks(self) -> bool {
        matches!(self, Self::Load | Self::Enable)
    }
}
