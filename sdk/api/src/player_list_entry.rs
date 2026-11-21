use std::marker::PhantomData;

use amml_abi::{AmmlNetworkTable, AmmlPlayerListEntry, AmmlStatus, AmmlStr, NETWORK_TEXT_NAME, NETWORK_TEXT_XUID};

use crate::{ActorUniqueId, SkinRef, Uuid};

/// One entry of a `PlayerListUpdate`, lent for the event. A Remove entry has only a uuid: the other reads give `None`.
#[derive(Clone, Copy)]
pub struct PlayerListEntry<'f> {
    table: AmmlNetworkTable,
    index: u32,
    entry: AmmlPlayerListEntry,
    _event: PhantomData<&'f ()>,
}

impl<'f> PlayerListEntry<'f> {
    pub(crate) fn new(table: AmmlNetworkTable, index: u32, entry: AmmlPlayerListEntry) -> Self {
        Self { table, index, entry, _event: PhantomData }
    }

    /// The uuid of the player.
    pub fn uuid(&self) -> Uuid {
        Uuid(self.entry.uuid)
    }

    /// The actor unique id of the player; `None` for a Remove entry.
    pub fn actor_id(&self) -> Option<ActorUniqueId> {
        (self.entry.has_actor != 0).then_some(ActorUniqueId(self.entry.actor_id))
    }

    /// The name of the player; `None` for a Remove entry or a name that is not UTF-8.
    pub fn name(&self) -> Option<&'f str> {
        self.text(NETWORK_TEXT_NAME)
    }

    /// The Xbox user id of the player; `None` for a Remove entry or a text that is not UTF-8.
    pub fn xuid(&self) -> Option<&'f str> {
        self.text(NETWORK_TEXT_XUID)
    }

    /// The skin of the player; `None` for a Remove entry and for an Add entry that has no skin.
    pub fn skin(&self) -> Option<SkinRef<'f>> {
        (self.entry.skin != 0).then(|| SkinRef::new(self.table, self.entry.skin))
    }

    #[allow(unsafe_code)]
    fn text(&self, field: u8) -> Option<&'f str> {
        let mut out = AmmlStr::EMPTY;
        // SAFETY: the table function is the loader's; `out` is writable.
        let status = unsafe { (self.table.player_list_text)(self.index, field, &raw mut out) };
        if status != AmmlStatus::OK {
            return None;
        }
        // SAFETY: the loader keeps the text readable until the event ends, and `'f` ends with the borrow of the event.
        unsafe { out.as_str() }.ok()
    }
}
