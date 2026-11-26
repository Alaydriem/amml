use amml_runtime::ResourceId;

use crate::InputHold;

/// The input service's own bookkeeping: the live leases with their owners, the leases ended since
/// the last frame (their queued ups still need an owner), the holds, and the mods whose leases
/// were ended for a disable (they get no new lease until they are enabled again), and the leases
/// whose holds a real up ended since the last frame.
#[derive(Debug, Default)]
pub struct InputLedger {
    pub leases: Vec<(ResourceId, usize)>,
    pub retiring: Vec<(ResourceId, usize)>,
    pub holds: Vec<InputHold>,
    pub closed: Vec<usize>,
    pub reset: Vec<ResourceId>,
}
