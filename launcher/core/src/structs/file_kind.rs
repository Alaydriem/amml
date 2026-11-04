// The kinds of file a registry serves, each with the size it may have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Root,
    Names,
    IndexPage,
    ModDocument,
    Icon,
}

impl FileKind {
    pub fn limit(&self) -> u64 {
        match self {
            Self::Root => 64 * 1024,
            Self::Names => 2 * 1024 * 1024,
            Self::IndexPage => 256 * 1024,
            Self::ModDocument => 256 * 1024,
            Self::Icon => 64 * 1024,
        }
    }
}
