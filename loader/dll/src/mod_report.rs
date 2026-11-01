use crate::{ModLoadError, ModPhase};

/// What happened to one mod in a load or enable pass.
#[derive(Debug)]
pub struct ModReport {
    pub name: String,
    pub phase: ModPhase,
    pub error: Option<ModLoadError>,
}

impl ModReport {
    /// Whether the mod is still usable after this pass.
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }
}
