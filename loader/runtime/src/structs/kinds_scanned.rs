use crate::structs::KindsOutcome;

/// The answer of `KindsScan::scan` and the number of cells it read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KindsScanned {
    pub outcome: KindsOutcome,
    pub cells: u32,
}
