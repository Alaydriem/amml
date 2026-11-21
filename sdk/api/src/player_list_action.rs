use amml_abi::PlayerListUpdate;

/// What a player list update does with its entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerListAction {
    /// The entries add players; each has an actor id, a name, an Xbox user id and, usually, a skin.
    Add,
    /// The entries remove players; each has only a uuid.
    Remove,
}

impl From<u32> for PlayerListAction {
    fn from(action: u32) -> Self {
        if action == PlayerListUpdate::REMOVE { Self::Remove } else { Self::Add }
    }
}

impl From<PlayerListAction> for u32 {
    fn from(action: PlayerListAction) -> Self {
        match action {
            PlayerListAction::Add => PlayerListUpdate::ADD,
            PlayerListAction::Remove => PlayerListUpdate::REMOVE,
        }
    }
}
