use amml_abi::{AmmlActorTable, AmmlClientTable, AmmlInventoryTable};

/// The loader tables a `Client` and its `LocalPlayer` call: the client table, the actor table when the services
/// table has one (`size >= SIZE_V4`) and the inventory table (`size >= SIZE_V5`). Copied once by `Ctx::from_raw`.
#[derive(Clone, Copy)]
pub(crate) struct ClientTables {
    pub(crate) client: AmmlClientTable,
    pub(crate) actor: Option<AmmlActorTable>,
    pub(crate) inventory: Option<AmmlInventoryTable>,
}
