use std::path::PathBuf;

// A copy of `Minecraft.Windows.exe` that was made and checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExeCopy {
    pub path: PathBuf,
    pub bytes: u64,
    // `"plain"` or `"package-context"`
    pub method: &'static str,
}
