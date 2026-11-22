use amml_abi::AmmlOtherLeases;

use crate::InputBlocks;

/// The active input leases of the other mods (`Input::held_by_others`): how many, and the union of the real input
/// they block. The calling mod's own leases never count, and a suspended lease does not count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OtherLeases {
    pub leases: u32,
    pub blocks: InputBlocks,
}

impl OtherLeases {
    /// Whether another mod holds an active lease (also one that blocks no real input).
    pub fn any(&self) -> bool {
        self.leases > 0
    }
}

impl From<AmmlOtherLeases> for OtherLeases {
    fn from(leases: AmmlOtherLeases) -> Self {
        Self { leases: leases.leases, blocks: leases.blocks.into() }
    }
}
