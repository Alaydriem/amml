/// Where a `FakeGame` is in the mod lifecycle, as its driver set it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FakePhase {
    /// No driver call yet: `listen` and `provide` are accepted at any time, for tests of one piece
    /// of a mod.
    Free,
    /// Inside the mod's `load` or `enable`: `listen` and `provide` are accepted.
    Registering,
    /// Between or after the callbacks, as in the game: `listen` and `provide` give `FAILED`.
    Running,
}
