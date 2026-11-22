/// Why `BlockTagWriter` wrote no tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BlockTagError {
    #[error("a text of {len} bytes does not fit the 16-bit length of an NBT string")]
    TooLong { len: usize },
}
