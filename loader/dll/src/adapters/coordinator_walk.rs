use std::ffi::c_void;

use amml_msvc::MsvcListNode;

use crate::LoaderState;

/// `void RenderChunkCoordinator::_setDirty(BlockPos const& min, BlockPos const& max, bool immediate, bool
/// changesVisibility, bool canInterlock)`.
type SetDirtyFn = unsafe extern "C" fn(*mut c_void, *const [i32; 3], *const [i32; 3], bool, bool, bool);

/// One node value of `LevelRenderer.mRenderChunkCoordinators`, an
/// `unordered_map<DimensionType, shared_ptr<RenderChunkCoordinator>>`: the key (padded to 8) and the
/// shared pointer (object, control block).
#[repr(C)]
#[derive(Clone, Copy)]
struct CoordinatorPair {
    dimension: i32,
    _pad: u32,
    coordinator: *mut c_void,
    control: *mut c_void,
}

/// Marks terrain meshes dirty through the game's own `RenderChunkCoordinator::_setDirty`, the call the
/// coordinator's block-change listener makes; the frame's `LevelRenderer` update then rebuilds them
/// (within 24 blocks of the camera at once, with `immediate`). A map node is used only when its object has
/// the vtable of `RenderChunkCoordinator.vtable`.
pub struct CoordinatorWalk {
    set_dirty: SetDirtyFn,
    vtable: usize,
    map: usize,
}

impl CoordinatorWalk {
    const SET_DIRTY: &'static str = "RenderChunkCoordinator._setDirty";
    const VTABLE: &'static str = "RenderChunkCoordinator.vtable";
    const MAP: &'static str = "LevelRenderer.mRenderChunkCoordinators";
    // More nodes than dimensions exist; a longer walk is not this map.
    const MAX_NODES: usize = 16;

    /// The walk over the three entries; `None` when one is missing.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let set_dirty = value(Self::SET_DIRTY).filter(|address| *address != 0)?;
        // SAFETY: the entry is the start of `RenderChunkCoordinator::_setDirty`, whose signature is `SetDirtyFn`.
        let set_dirty = unsafe { std::mem::transmute::<usize, SetDirtyFn>(set_dirty) };
        Some(Self { set_dirty, vtable: value(Self::VTABLE)?, map: value(Self::MAP)? })
    }

    /// Marks each box (inclusive block min, max) dirty in every coordinator that passes the vtable guard;
    /// gives the number of calls. A coordinator drops marks for subchunks it has no render chunk for, so the
    /// coordinators of other dimensions take no harm.
    ///
    /// # Safety
    /// `level_renderer` is the live `LevelRenderer`; on the thread of its update.
    pub unsafe fn mark(&self, level_renderer: *mut c_void, boxes: &[([i32; 3], [i32; 3])]) -> Result<usize, String> {
        if level_renderer.is_null() {
            return Err("no level renderer".to_string());
        }
        // SAFETY: as the caller guarantees.
        let (coordinators, refused) = unsafe { self.coordinators(level_renderer) };
        if coordinators.is_empty() {
            return Err(format!("no coordinator passed the vtable guard ({refused} refused)"));
        }
        let mut calls = 0;
        for coordinator in coordinators {
            for (min, max) in boxes {
                // SAFETY: a guarded coordinator and two positions that live across the call.
                unsafe { (self.set_dirty)(coordinator, min, max, true, false, false) };
                calls += 1;
            }
        }
        Ok(calls)
    }

    // The coordinators that pass the guard, and how many nodes failed it.
    //
    // Safety: as `mark`.
    unsafe fn coordinators(&self, level_renderer: *mut c_void) -> (Vec<*mut c_void>, usize) {
        let mut found = Vec::new();
        let mut refused = 0;
        // SAFETY: the live LevelRenderer holds the list head pointer of the map at the entry offset.
        let head =
            unsafe { level_renderer.cast::<u8>().add(self.map).cast::<*const MsvcListNode<CoordinatorPair>>().read() };
        if head.is_null() {
            return (found, refused);
        }
        // SAFETY: the head is the sentinel node of the map's list; each `next` is a live node or the head.
        let mut node = unsafe { (*head).next() };
        for _ in 0..Self::MAX_NODES {
            if node.is_null() || node == head {
                break;
            }
            // SAFETY: a live node of the list.
            let pair = unsafe { *(*node).value() };
            if !pair.coordinator.is_null() {
                // SAFETY: a live object whose first qword is its vtable.
                let vtable = unsafe { pair.coordinator.cast::<usize>().read() };
                if vtable == self.vtable {
                    found.push(pair.coordinator);
                } else {
                    refused += 1;
                }
            }
            // SAFETY: as above.
            node = unsafe { (*node).next() };
        }
        (found, refused)
    }
}
