use super::FieldKind;

/// One decoded instruction and which of its bytes are immediate, displacement or branch fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoveredInstruction {
    pub ip: u32,
    pub len: usize,
    pub text: String,
    // (kind, offset from the instruction start, size)
    pub fields: Vec<(FieldKind, usize, usize)>,
}
