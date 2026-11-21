use amml_abi::LookDelta;

use crate::Look;

/// Typed reads and writes of a `LookDelta`.
pub trait LookDeltaExt {
    fn delta(&self) -> Look;
    /// Changes the turn the game applies; later listeners see the new value.
    fn set_delta(&mut self, delta: Look);
}

impl LookDeltaExt for LookDelta {
    fn delta(&self) -> Look {
        Look { yaw: self.yaw, pitch: self.pitch }
    }

    fn set_delta(&mut self, delta: Look) {
        self.yaw = delta.yaw;
        self.pitch = delta.pitch;
    }
}
