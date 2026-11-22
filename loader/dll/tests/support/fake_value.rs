use super::FakeCompound;

/// The value of one entry of a fake compound tag.
pub enum FakeValue {
    Byte(i8),
    Int(i32),
    /// A string tag.
    Text(String),
    /// A nested compound tag, held in the node.
    Compound(FakeCompound),
    List(Vec<FakeCompound>),
    /// A list whose first element is a byte tag, as a malformed save has it.
    ListWithAByte(Vec<FakeCompound>),
}
