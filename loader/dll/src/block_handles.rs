use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlBlockHandle, AmmlStatus};
use amml_runtime::{ResourceError, ResourceId, ResourceKind};

use crate::LoaderState;

static HANDLES: OnceLock<BlockHandles> = OnceLock::new();

/// The `Block const*` the loader holds for mods, as owned resources of kind `BLOCK` in the resource store:
/// bound to the mod and the world, so they are `STALE` after a disable or a world leave. The same block
/// for the same mod in the same world gives the same handle, so a mod can compare them and a per-tick read
/// takes no new slot.
pub struct BlockHandles {
    interned: Mutex<HashMap<(usize, usize), ResourceId>>,
}

impl BlockHandles {
    pub const BLOCK: ResourceKind = ResourceKind(5);

    fn get() -> &'static BlockHandles {
        HANDLES.get_or_init(|| BlockHandles { interned: Mutex::new(HashMap::new()) })
    }

    // The map holds plain ids, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, HashMap<(usize, usize), ResourceId>> {
        self.interned.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The handle of the block at `address` for the mod at `owner`, in the current world. `NOT_READY`
    /// with no world joined.
    pub(crate) fn handle(state: &LoaderState, owner: usize, address: usize) -> Result<AmmlBlockHandle, AmmlStatus> {
        let epoch = state.events().gate().epoch();
        if epoch.0 == 0 {
            return Err(AmmlStatus::NOT_READY);
        }
        let mut interned = Self::get().lock();
        if let Some(id) = interned.get(&(owner, address))
            && state.resources().get_kind(*id, Self::BLOCK) == Ok(address)
        {
            return Ok(AmmlBlockHandle(id.0));
        }
        match state.resources().insert(Self::BLOCK, owner, epoch, address) {
            Ok(id) => {
                interned.insert((owner, address), id);
                Ok(AmmlBlockHandle(id.0))
            }
            Err((error, _)) => Err(Self::status(error)),
        }
    }

    /// The block address of `handle`; `STALE` when the handle was taken back, `INVALID_ARGUMENT` for a
    /// handle of another kind.
    pub(crate) fn address(state: &LoaderState, handle: AmmlBlockHandle) -> Result<usize, AmmlStatus> {
        state.resources().get_kind(ResourceId(handle.0), Self::BLOCK).map_err(Self::status)
    }

    fn status(error: ResourceError) -> AmmlStatus {
        match error {
            ResourceError::WrongKind => AmmlStatus::INVALID_ARGUMENT,
            _ => AmmlStatus::STALE,
        }
    }
}
