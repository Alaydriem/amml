/// Whether the host runs the game heap round trip while it starts the loader.
///
/// The check calls the game's `operator new` and `operator delete`. That is safe only once the game
/// has run its static initializers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeapCheck {
    /// Check at load. The LeviLamina host: the game is initialized when LeviLamina loads a mod.
    AtLoad,
    /// Check on the first allocation a mod makes. The standalone host: mod load and enable run
    /// before the game's static initializers, so the game allocator may not be set up yet, and mods
    /// must not call game code (the game heap included) in load or enable.
    Deferred,
}
