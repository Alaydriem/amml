use amml_abi::InventoryRequestDone;

use crate::{RequestId, RequestOutcome};

/// Typed reads of an `InventoryRequestDone`.
pub trait InventoryRequestDoneExt {
    /// The id `Inventory::request_moves` gave.
    fn id(&self) -> RequestId;

    /// How the request ended.
    fn outcome(&self) -> RequestOutcome;

    /// The number of moves the server accepted.
    fn accepted(&self) -> u16;

    /// The number of moves of the request.
    fn total(&self) -> u16;
}

impl InventoryRequestDoneExt for InventoryRequestDone {
    fn id(&self) -> RequestId {
        RequestId(self.id)
    }

    fn outcome(&self) -> RequestOutcome {
        RequestOutcome::from(self.outcome)
    }

    fn accepted(&self) -> u16 {
        self.accepted
    }

    fn total(&self) -> u16 {
        self.total
    }
}
