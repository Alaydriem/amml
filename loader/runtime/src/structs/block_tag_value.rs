/// The value of one block state in a block tag that `BlockTagWriter` writes: an int (`TAG_Int`), a bool (`TAG_Byte` 0
/// or 1) or a text (`TAG_String`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockTagValue<'a> {
    Int(i32),
    Bool(bool),
    Text(&'a str),
}
