use amml_abi::{AmmlInventoryMove, AmmlStatus};

/// One move of `Inventory::request_moves`, between the slots of the player inventory (0 to 35: 0 to 8 the hotbar, 9
/// to 35 the main inventory).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryMove {
    /// The stacks of slots `a` and `b` change places; both slots hold a stack.
    Swap { a: u8, b: u8 },
    /// `count` items of slot `from` go to slot `to`, which is empty or holds a stack of the same item.
    Place { from: u8, to: u8, count: u8 },
    /// `count` items of slot `from` are thrown out.
    Drop { from: u8, count: u8 },
}

impl From<InventoryMove> for AmmlInventoryMove {
    fn from(value: InventoryMove) -> Self {
        match value {
            InventoryMove::Swap { a, b } => Self { kind: Self::SWAP, a, b, count: 0 },
            InventoryMove::Place { from, to, count } => Self { kind: Self::PLACE, a: from, b: to, count },
            InventoryMove::Drop { from, count } => Self { kind: Self::DROP, a: from, b: 0, count },
        }
    }
}

impl TryFrom<AmmlInventoryMove> for InventoryMove {
    type Error = AmmlStatus;

    /// The move of the ABI struct; `NOT_AVAILABLE` for the reserved pick kind, `INVALID_ARGUMENT` for another kind.
    fn try_from(value: AmmlInventoryMove) -> Result<Self, AmmlStatus> {
        match value.kind {
            AmmlInventoryMove::SWAP => Ok(Self::Swap { a: value.a, b: value.b }),
            AmmlInventoryMove::PLACE => Ok(Self::Place { from: value.a, to: value.b, count: value.count }),
            AmmlInventoryMove::DROP => Ok(Self::Drop { from: value.a, count: value.count }),
            AmmlInventoryMove::PICK => Err(AmmlStatus::NOT_AVAILABLE),
            _ => Err(AmmlStatus::INVALID_ARGUMENT),
        }
    }
}
