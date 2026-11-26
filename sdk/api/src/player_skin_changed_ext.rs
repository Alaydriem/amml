use amml_abi::{AmmlNetworkTable, AmmlStatus, PlayerSkinChanged};

use crate::lent_network::LentNetwork;
use crate::{SkinRef, Uuid};

/// Typed reads of a `PlayerSkinChanged`.
pub trait PlayerSkinChangedExt {
    /// The uuid of the player whose skin changed; `None` for a `PlayerSkinChanged` that is not the event of a running
    /// listener call on this thread (a copy, or one kept past its call), and when the loader has no network table.
    fn uuid(&self) -> Option<Uuid>;

    /// The new skin, under the same rule as `uuid`; `None` too when the packet holds no skin.
    fn skin(&self) -> Option<SkinRef<'_>>;
}

impl PlayerSkinChangedExt for PlayerSkinChanged {
    fn uuid(&self) -> Option<Uuid> {
        read(self).map(|(uuid, _, _)| uuid)
    }

    fn skin(&self) -> Option<SkinRef<'_>> {
        let (_, handle, table) = read(self)?;
        (handle != 0).then(|| SkinRef::new(table, handle))
    }
}

// The uuid, the skin handle and the table of the running event `event`.
#[allow(unsafe_code)]
fn read(event: &PlayerSkinChanged) -> Option<(Uuid, u64, AmmlNetworkTable)> {
    let table = LentNetwork::find(std::ptr::from_ref(event))?;
    let (mut uuid, mut skin) = ([0u8; 16], 0u64);
    // SAFETY: the table function is the loader's; both outputs are writable.
    let status = unsafe { (table.player_skin)(&raw mut uuid, &raw mut skin) };
    (status == AmmlStatus::OK).then_some((Uuid(uuid), skin, table))
}
