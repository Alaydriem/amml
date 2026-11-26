use amml_abi::AmmlStatus;

/// The build's key path, as the input service calls it to inject a key.
pub trait KeyFeed: Send + Sync {
    /// The adapter shape behind the feed, for the log.
    fn shape(&self) -> &'static str;
    /// `Ok` when a key can be sent now; `NOT_READY` or `NOT_AVAILABLE` otherwise.
    fn ready(&self) -> Result<(), AmmlStatus>;
    /// Feeds one key to the game, past the loader's own detour. Game thread.
    fn send(&self, key: u16, down: bool);
    /// Raises a real up for every key the loader holds down, without sending it to the game, because the
    /// game resets its own keys. A feed that runs no real key path does nothing.
    fn release_real(&self) {}
}
