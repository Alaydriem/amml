/// The build's turn path, as the input service calls it to turn the local player's view.
pub trait TurnFeed: Send + Sync {
    /// Turns the view by `yaw` and `pitch` degrees. False when there is no local player: the turn
    /// is dropped. Game thread.
    fn send_turn(&self, yaw: f32, pitch: f32) -> bool;
}
