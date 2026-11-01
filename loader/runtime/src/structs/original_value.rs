/// What one `original` slot must hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginalValue {
    /// The detour of the next hook in the chain.
    Hook(u64),
    /// The trampoline, which is the original function: the hook is the last in the chain.
    Trampoline,
}
