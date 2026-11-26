use crate::testing::{FakePlayerEntry, FakeSkin};
use crate::{PlayerListAction, Uuid};

// The event running now: the entries of a player list update, or the player of a skin change.
pub(crate) struct FakeNetworkRun {
    pub(crate) serial: u64,
    pub(crate) list: Option<(PlayerListAction, Vec<FakePlayerEntry>)>,
    pub(crate) skin_event: Option<(Uuid, Option<FakeSkin>)>,
}
