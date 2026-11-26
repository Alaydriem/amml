use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::structs::material_patch_state::MaterialPatchState;
use crate::{MaterialBytes, MaterialChoice, MaterialPlan, MaterialWrite};

/// The patch of the game's `wireframe` material for Vibrant Visuals world meshes: alpha blend, no depth write.
///
/// The patch makes no game call; it says what the render thread writes. The first patch keeps the four bytes the
/// material had. A restore writes them back once, and only into the material the running level renderer has at that
/// moment: a kept material at another address (the renderer of a left world) is forgotten, never written. A restore
/// is asked for (`request_restore`, `disable_owner`) from any thread and written by the render thread
/// (`take_restore`, or the next `prepare`). A draw in another graphics mode first restores the kept bytes, then uses
/// the name tag material.
pub struct MaterialPatch {
    state: Mutex<MaterialPatchState>,
    // True while bytes are kept; the render detour reads it without the lock.
    applied: AtomicBool,
}

impl Default for MaterialPatch {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialPatch {
    /// The bytes of the patched material: blend on, `SourceAlpha` (6) and `OneMinusSrcAlpha` (8), depth write off.
    pub const PATCHED: MaterialBytes =
        MaterialBytes { enable_blend: 1, blend_source: 6, blend_destination: 8, depth_write: 0 };

    pub fn new() -> Self {
        Self {
            state: Mutex::new(MaterialPatchState { kept: None, restore_due: false, drawers: BTreeSet::new() }),
            applied: AtomicBool::new(false),
        }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, MaterialPatchState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Before one draw of a mesh of `owner`, on the render thread. `vibrant` is the graphics mode; `wireframe` is the
    /// address of the running renderer's wireframe material and the bytes it has now (`None` when the game gave none).
    /// The plan writes a restore that is due, or that a draw in another mode needs, before anything else; for Vibrant
    /// it then patches the material (keeping its bytes at the first patch).
    pub fn prepare(&self, owner: usize, vibrant: bool, wireframe: Option<(usize, MaterialBytes)>) -> MaterialPlan {
        let mut state = self.lock();
        let current = wireframe.map(|(address, _)| address);
        let mut writes = Vec::new();
        if (state.restore_due || !vibrant)
            && let Some(restore) = Self::restore(&mut state, current)
        {
            writes.push(restore);
        }
        if state.kept.is_some_and(|kept| Some(kept.address) != current) {
            state.kept = None;
            state.drawers.clear();
        }
        if !vibrant {
            self.applied.store(state.kept.is_some(), Ordering::Release);
            return MaterialPlan { writes, choice: MaterialChoice::NameTag };
        }
        if let Some((address, bytes)) = wireframe {
            let kept_bytes = writes.last().map_or(bytes, |restore| restore.bytes);
            if state.kept.is_none() {
                state.kept = Some(MaterialWrite { address, bytes: kept_bytes });
            }
            writes.push(MaterialWrite { address, bytes: Self::PATCHED });
            state.drawers.insert(owner);
        }
        self.applied.store(state.kept.is_some(), Ordering::Release);
        MaterialPlan { writes, choice: MaterialChoice::Wireframe }
    }

    /// Asks for the kept bytes to be written back at the next render call: a world leave, a graphics mode switch, a
    /// render device reset, the shutdown.
    pub fn request_restore(&self) {
        let mut state = self.lock();
        if state.kept.is_some() {
            state.restore_due = true;
        }
    }

    /// The mod at `owner` is disabled. When it was the last mod that drew with the patched material, the restore is
    /// due.
    pub fn disable_owner(&self, owner: usize) {
        let mut state = self.lock();
        if state.drawers.remove(&owner) && state.drawers.is_empty() && state.kept.is_some() {
            state.restore_due = true;
        }
    }

    /// On the render thread, with the address of the running renderer's wireframe material (`None` when there is
    /// none): the restore write when one is due and the material is the kept one. The kept bytes are forgotten when
    /// a restore is due, written or not, so the bytes are written at most once.
    pub fn take_restore(&self, current: Option<usize>) -> Option<MaterialWrite> {
        let mut state = self.lock();
        if !state.restore_due {
            return None;
        }
        let restore = Self::restore(&mut state, current);
        self.applied.store(false, Ordering::Release);
        restore
    }

    /// Whether bytes are kept: the material is patched, or was and waits for its restore. One atomic load.
    pub fn is_applied(&self) -> bool {
        self.applied.load(Ordering::Acquire)
    }

    /// Whether a restore was asked for and is not written yet.
    pub fn restore_due(&self) -> bool {
        self.lock().restore_due
    }

    fn restore(state: &mut MaterialPatchState, current: Option<usize>) -> Option<MaterialWrite> {
        state.restore_due = false;
        state.drawers.clear();
        state.kept.take().filter(|kept| Some(kept.address) == current)
    }
}
