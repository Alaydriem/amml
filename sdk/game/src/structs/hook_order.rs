/// Hook priorities that order mods sharing one game function. A higher priority runs first.
pub struct HookOrder;

impl HookOrder {
    /// For a mod that changes what the game reads or writes.
    pub const WRITE: i32 = 0;
    /// For a mod that reads state after every writer has run.
    pub const READ_AFTER_WRITERS: i32 = 100;
}
