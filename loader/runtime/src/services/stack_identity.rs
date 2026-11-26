/// What makes two item stacks the same kind to the game's merge rule, and the 64-bit identity made from it.
///
/// The identity is FNV-1a 64 of the type name, one zero byte (a type name has none), the aux value as two
/// little-endian bytes and the user data bytes. The fixed-size parts before the user data keep two different
/// triples from making the same bytes. The count is not part of it: two stacks the game merges have equal
/// identity. The value is the same in every run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StackIdentity<'a> {
    pub type_name: &'a str,
    pub aux: i16,
    /// Kept for the caller's view of the stack; it does not change the identity.
    pub count: u8,
    /// The bytes `CompoundTag::write` gives for the stack's user data; empty for a stack without user data.
    pub user_data: &'a [u8],
}

impl StackIdentity<'_> {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    /// The identity value.
    pub fn value(&self) -> u64 {
        let parts = [self.type_name.as_bytes(), &[0], &self.aux.to_le_bytes(), self.user_data];
        parts
            .iter()
            .flat_map(|part| part.iter())
            .fold(Self::OFFSET_BASIS, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(Self::PRIME))
    }
}
