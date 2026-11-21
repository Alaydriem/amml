/// A view turn in degrees: positive yaw turns right as seen from above, positive pitch looks down.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Look {
    pub yaw: f32,
    pub pitch: f32,
}
