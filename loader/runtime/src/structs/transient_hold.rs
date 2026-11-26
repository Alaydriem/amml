/// One transient option value: the key, the game option id the first transient set wrote, the mod that set it, the
/// value the option held before the first transient set and the value the mod set last.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransientHold {
    pub key: u32,
    pub option: i32,
    pub owner: usize,
    pub before: i32,
    pub transient: i32,
}
