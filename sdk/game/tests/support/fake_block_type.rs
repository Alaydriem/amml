use super::fake_hashed_string::FakeHashedString;

/// A fake `BlockType`: the bytes before the full name, then the full-name `HashedString` at E0h, its place on 26.51
/// and 26.60.
#[allow(dead_code)]
#[repr(C)]
pub struct FakeBlockType {
    pub pad: [u8; 0xE0],
    pub name: FakeHashedString,
}
