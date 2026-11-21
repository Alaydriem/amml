/// Why the world mesh path is not offered on this build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MeshLayoutError {
    #[error("the entry {entry} does not resolve")]
    Missing { entry: &'static str },
    #[error("the entry {entry} is {found:#x}, the mirror needs {expected:#x}")]
    Mismatch { entry: &'static str, expected: u64, found: u64 },
}

impl MeshLayoutError {
    /// The entry the error names.
    pub fn entry(&self) -> &'static str {
        match self {
            Self::Missing { entry } | Self::Mismatch { entry, .. } => entry,
        }
    }
}
