use amml_abi::AmmlMouseButton;

use crate::structs::ResourceId;

/// The holds of one lease.
///
/// `wanted` is what the mod holds: downs queued or applied, with no up since.
/// `applied` is what the game holds: downs taken by a frame, with no up taken since.
pub(crate) struct LeaseInput {
    pub(crate) lease: ResourceId,
    pub(crate) wanted_keys: Vec<u16>,
    pub(crate) applied_keys: Vec<u16>,
    pub(crate) wanted_buttons: Vec<AmmlMouseButton>,
    pub(crate) applied_buttons: Vec<AmmlMouseButton>,
    /// The wheel notches below one whole notch, kept for the next call.
    pub(crate) wheel_rest: f32,
}
