use amml_abi::{AmmlPlayerListEntry, AmmlStatus, PlayerListUpdate};

use crate::lent_network::LentNetwork;
use crate::{PlayerListAction, PlayerListEntry};

/// Typed reads of a `PlayerListUpdate`.
pub trait PlayerListUpdateExt {
    /// Whether the entries add or remove players.
    fn action(&self) -> PlayerListAction;

    /// The entries of the update, at most 4096, as views of the game's packet. Empty when the loader has no network
    /// table, and for a `PlayerListUpdate` that is not the event of a running listener call on this thread (a copy, or
    /// one kept past its call). An entry whose read fails is left out.
    fn entries(&self) -> impl Iterator<Item = PlayerListEntry<'_>>;
}

impl PlayerListUpdateExt for PlayerListUpdate {
    fn action(&self) -> PlayerListAction {
        PlayerListAction::from(self.action)
    }

    #[allow(unsafe_code)]
    fn entries(&self) -> impl Iterator<Item = PlayerListEntry<'_>> {
        let table = LentNetwork::find(std::ptr::from_ref(self));
        let count = if table.is_some() { self.count.min(PlayerListUpdate::MAX_ENTRIES) } else { 0 };
        (0..count).filter_map(move |index| {
            let table = table?;
            let mut out = AmmlPlayerListEntry { uuid: [0; 16], has_actor: 0, _reserved: [0; 7], actor_id: 0, skin: 0 };
            // SAFETY: the table function is the loader's; `out` is writable.
            let status = unsafe { (table.player_list_entry)(index, &raw mut out) };
            (status == AmmlStatus::OK).then(|| PlayerListEntry::new(table, index, out))
        })
    }
}
