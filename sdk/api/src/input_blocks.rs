use std::ops::BitOr;

use amml_abi::AmmlInputBlocks;

/// The kinds of real input a lease blocks while it is active, as a bit set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct InputBlocks(u32);

impl InputBlocks {
    pub const NONE: Self = Self(0);
    pub const KEYS: Self = Self(AmmlInputBlocks::KEYS.0);
    pub const MOUSE_BUTTONS: Self = Self(AmmlInputBlocks::MOUSE_BUTTONS.0);
    pub const LOOK: Self = Self(AmmlInputBlocks::LOOK.0);
    pub const MOVE: Self = Self(AmmlInputBlocks::MOVE.0);
    pub const GAMEPAD: Self = Self(AmmlInputBlocks::GAMEPAD.0);
    pub const FULL: Self = Self(AmmlInputBlocks::FULL.0);

    /// Whether every bit of `other` is set.
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for InputBlocks {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl From<AmmlInputBlocks> for InputBlocks {
    fn from(blocks: AmmlInputBlocks) -> Self {
        Self(blocks.0 & AmmlInputBlocks::FULL.0)
    }
}

impl From<InputBlocks> for AmmlInputBlocks {
    fn from(blocks: InputBlocks) -> Self {
        Self(blocks.0)
    }
}
