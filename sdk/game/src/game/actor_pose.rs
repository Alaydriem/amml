/// Where an actor is and where it looks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorPose {
    /// The position: x, y, z.
    pub pos: [f32; 3],
    /// The look angle up and down, in degrees.
    pub pitch: f32,
    /// The look angle around, in degrees.
    pub yaw: f32,
}
