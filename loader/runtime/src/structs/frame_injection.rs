use amml_abi::AmmlMoveIntent;

use crate::structs::{InjectedOp, ResourceId};

/// The injected input of one frame, in the order the leases queued it.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameInjection {
    pub ops: Vec<(ResourceId, InjectedOp)>,
    /// The turns of all leases since the last frame, summed; `None` when there were none.
    pub turn: Option<(f32, f32)>,
    /// The move intent in force; it stays until a lease changes it.
    pub move_intent: Option<(ResourceId, AmmlMoveIntent)>,
}
