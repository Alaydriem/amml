/// A click in a player row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerRowAction {
    /// Start following this player, or stop.
    ToggleFollow,
    /// Move the camera to this player.
    GoTo,
}
