use amml_abi::AmmlBlockHandle;

/// A block the loader holds for this mod (from `ItemView::block`). It names no game memory: the loader looks it
/// up on each use. The same block in the same world gives an equal handle. After this mod's disable or a world
/// leave it is stale, and a call that takes it gives `ApiError::Stale`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockHandle(AmmlBlockHandle);

impl BlockHandle {
    /// The handle of a non-zero loader id; `None` for `AmmlBlockHandle::NONE`.
    pub(crate) fn from_raw(raw: AmmlBlockHandle) -> Option<Self> {
        (raw != AmmlBlockHandle::NONE).then_some(Self(raw))
    }

    pub(crate) fn raw(self) -> AmmlBlockHandle {
        self.0
    }
}
