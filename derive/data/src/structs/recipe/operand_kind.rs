#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperandKind {
    Displacement,
    Immediate,
    RipTarget,
    BranchTarget,
    /// The multiplier of the index of a memory operand: its scale, plus 1 when the base is the index register too
    /// (`lea rax,[rdx+rdx*8]` multiplies by 9). A stride that the compiler builds from a lea and a shift.
    ScaleFactor,
}

impl OperandKind {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "displacement" => Some(Self::Displacement),
            "immediate" => Some(Self::Immediate),
            "rip_target" => Some(Self::RipTarget),
            "branch_target" => Some(Self::BranchTarget),
            "scale_factor" => Some(Self::ScaleFactor),
            _ => None,
        }
    }

    /// The word that recipes use for this kind.
    pub fn name(self) -> &'static str {
        match self {
            Self::Displacement => "displacement",
            Self::Immediate => "immediate",
            Self::RipTarget => "rip_target",
            Self::BranchTarget => "branch_target",
            Self::ScaleFactor => "scale_factor",
        }
    }
}
