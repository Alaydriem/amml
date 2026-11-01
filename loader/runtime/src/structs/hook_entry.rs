/// One hook in a chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HookEntry {
    pub handle: u64,
    pub priority: i32,
    pub detour: u64,
    pub original_slot: u64,
}
