use amml_abi::{AmmlMouseButton, AmmlStatus};

/// The build's mouse path, as the input service calls it to inject a button or the wheel.
pub trait MouseFeed: Send + Sync {
    /// The adapter shape behind the feed, for the log.
    fn shape(&self) -> &'static str;
    /// `Ok` when a button or wheel can be sent now; `NOT_READY` or `NOT_AVAILABLE` otherwise.
    fn ready(&self) -> Result<(), AmmlStatus>;
    /// Feeds one button to the game, past the loader's own detour. Game thread.
    fn send_button(&self, button: AmmlMouseButton, down: bool);
    /// Feeds whole wheel notches to the game, past the loader's own detour. Game thread.
    fn send_wheel(&self, notches: f32);
}
