/// How one instruction uses a member of the object: it reads it, writes it, or takes its address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccessKind {
    Read,
    Write,
    Lea,
}

impl AccessKind {
    pub fn label(self) -> &'static str {
        match self {
            AccessKind::Read => "read",
            AccessKind::Write => "write",
            AccessKind::Lea => "lea",
        }
    }
}

/// One `[this+disp]` access of a function: the access kind, the member offset, the width in bytes and the RVA of the
/// instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldAccessRow {
    pub kind: AccessKind,
    pub disp: u32,
    pub width: u8,
    pub rva: u32,
}
