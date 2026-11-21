use crate::{AmmlEvent, AmmlEventHeader};

/// A request of `request_moves` ended (`inventory.request_done`, After the end). `id` is the id `request_moves`
/// gave, `outcome` an `AmmlRequestOutcome` value, `accepted` the number of moves the server accepted and `total` the
/// number of moves of the request.
///
/// Raised at a client tick after the end, on the game thread, never inside `request_moves`. A world leave ends a
/// running request with `CANCELLED` and raises the event before the world is taken back. A mod disable ends its
/// request and raises no event.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InventoryRequestDone {
    pub header: AmmlEventHeader,
    pub id: u64,
    pub outcome: u8,
    pub _reserved: u8,
    pub accepted: u16,
    pub total: u16,
    pub _reserved2: u16,
}

impl AmmlEvent for InventoryRequestDone {
    const ID: &'static str = "inventory.request_done";
}

const _: () = assert!(core::mem::size_of::<InventoryRequestDone>() == 32);
