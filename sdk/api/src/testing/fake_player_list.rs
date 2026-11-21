use crate::PlayerListAction;
use crate::testing::FakePlayerEntry;

/// A player list update for `FakeGame::raise_player_list`: the action and the entries, of which the fake gives at most
/// 4096, as the loader does.
#[derive(Clone, Debug)]
pub struct FakePlayerList {
    pub action: PlayerListAction,
    pub entries: Vec<FakePlayerEntry>,
}
