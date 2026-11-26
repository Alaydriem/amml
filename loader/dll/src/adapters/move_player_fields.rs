/// The fields of a `MovePlayerPacket` the `PlayerMoved` event carries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovePlayerFields {
    pub player_id: u64,
    pub pos: [f32; 3],
    /// Pitch, then yaw, in degrees.
    pub rot: [f32; 2],
    pub head_yaw: f32,
    pub mode: u8,
}
