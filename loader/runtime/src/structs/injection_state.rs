use amml_abi::AmmlMoveIntent;

use crate::structs::{InjectedOp, LeaseInput, ResourceId};

/// The queued ops, the per-lease holds, the per-lease turns since the last frame and the move
/// intent with its owner.
pub(crate) struct InjectionState {
    pub(crate) pending: Vec<(ResourceId, InjectedOp)>,
    pub(crate) inputs: Vec<LeaseInput>,
    pub(crate) turns: Vec<(ResourceId, f32, f32)>,
    pub(crate) move_intent: Option<(ResourceId, usize, AmmlMoveIntent)>,
}
