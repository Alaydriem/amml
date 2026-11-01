#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    Function,
    VSlot,
    Field,
    Global,
}

impl EntryKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::VSlot => "vslot",
            Self::Field => "field",
            Self::Global => "global",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "function" => Some(Self::Function),
            "vslot" => Some(Self::VSlot),
            "field" => Some(Self::Field),
            "global" => Some(Self::Global),
            _ => None,
        }
    }
}
