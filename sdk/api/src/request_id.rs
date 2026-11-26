/// The id of one request of `Inventory::request_moves`. `InventoryRequestDone::id` carries it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);
