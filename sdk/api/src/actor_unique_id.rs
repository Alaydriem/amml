/// The unique id of an actor (the game's `ActorUniqueID`), as the player list gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActorUniqueId(pub i64);
