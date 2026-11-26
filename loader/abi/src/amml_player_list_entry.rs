/// One entry of a `PlayerListUpdate`, written by `AmmlNetworkTable::player_list_entry`.
///
/// `has_actor` is 1 for an Add entry (`actor_id` is its actor unique id) and 0 for a Remove entry, which carries only
/// `uuid`. `skin` is a lent skin handle for the skin functions of the table, or 0 when the entry has no skin; it is
/// valid only while the event that gave it runs.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlPlayerListEntry {
    pub uuid: [u8; 16],
    pub has_actor: u8,
    pub _reserved: [u8; 7],
    pub actor_id: i64,
    pub skin: u64,
}

const _: () = assert!(core::mem::size_of::<AmmlPlayerListEntry>() == 40);
