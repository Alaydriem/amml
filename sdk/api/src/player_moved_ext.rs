use amml_abi::PlayerMoved;

use crate::{Rotation, Vec3};

/// Typed reads of a `PlayerMoved`: the game handled a `MovePlayerPacket` of the local player. Raised After the game's
/// handler, on the game thread. The packets of other actors raise nothing.
pub trait PlayerMovedExt {
    /// The position the packet carries.
    fn pos(&self) -> Vec3;
    /// The view direction the packet carries, in degrees, with the signs of `Rotation` (the angle rule of
    /// `LocalPlayer::rotation`).
    fn rotation(&self) -> Rotation;
    /// The head yaw the packet carries, in degrees.
    fn head_yaw(&self) -> f32;
    /// The packet's mode byte.
    fn mode(&self) -> u8;
}

impl PlayerMovedExt for PlayerMoved {
    fn pos(&self) -> Vec3 {
        Vec3::from(self.pos)
    }

    fn rotation(&self) -> Rotation {
        Rotation::from(self.rotation)
    }

    fn head_yaw(&self) -> f32 {
        self.head_yaw
    }

    fn mode(&self) -> u8 {
        self.mode
    }
}
