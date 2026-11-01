/// How an instruction or a data word refers to an address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeRefKind {
    /// A direct near call.
    Call,
    /// A direct near jump, conditional or not.
    Jump,
    /// An instruction with a RIP-relative memory operand that names the address.
    RipRelative,
    /// An 8-byte absolute pointer in a non-executable section.
    DataPointer,
}

impl CodeRefKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Jump => "jump",
            Self::RipRelative => "rip-relative",
            Self::DataPointer => "data-pointer",
        }
    }
}
