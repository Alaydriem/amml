use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use amml_abi::AmmlPermissionLevel;

/// The permission level of a player. A value the enum does not name keeps the game's raw number in `Other`. The order
/// is the order of the raw values (`Operator` is above `Member`), and two levels with the same raw value are equal,
/// so `Other(1)` equals `Member`. `Operator` (2) and `Custom` (3) are read from the executable; `Visitor` (0) and
/// `Member` (1) are the protocol's order, not proven from the executable.
#[derive(Clone, Copy, Debug)]
pub enum PermissionLevel {
    /// Raw 0: the protocol's order, not read from the executable.
    Visitor,
    /// Raw 1: the protocol's order, not read from the executable.
    Member,
    Operator,
    Custom,
    Other(u8),
}

impl PermissionLevel {
    /// The game's raw value.
    pub fn raw(self) -> u8 {
        match self {
            Self::Visitor => AmmlPermissionLevel::VISITOR.0,
            Self::Member => AmmlPermissionLevel::MEMBER.0,
            Self::Operator => AmmlPermissionLevel::OPERATOR.0,
            Self::Custom => AmmlPermissionLevel::CUSTOM.0,
            Self::Other(value) => value,
        }
    }
}

impl PartialEq for PermissionLevel {
    fn eq(&self, other: &Self) -> bool {
        self.raw() == other.raw()
    }
}

impl Eq for PermissionLevel {}

impl Hash for PermissionLevel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw().hash(state);
    }
}

impl PartialOrd for PermissionLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PermissionLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw().cmp(&other.raw())
    }
}

impl From<AmmlPermissionLevel> for PermissionLevel {
    fn from(value: AmmlPermissionLevel) -> Self {
        match value {
            AmmlPermissionLevel::VISITOR => Self::Visitor,
            AmmlPermissionLevel::MEMBER => Self::Member,
            AmmlPermissionLevel::OPERATOR => Self::Operator,
            AmmlPermissionLevel::CUSTOM => Self::Custom,
            AmmlPermissionLevel(other) => Self::Other(other),
        }
    }
}

impl From<PermissionLevel> for AmmlPermissionLevel {
    fn from(value: PermissionLevel) -> Self {
        AmmlPermissionLevel(value.raw())
    }
}
