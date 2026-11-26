/// A view direction in degrees. `yaw` 0 faces +Z (south) and grows clockwise seen from above (90 faces -X, west);
/// `pitch` 0 is level and grows looking down (90 straight down, -90 straight up).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct AmmlRotation {
    pub yaw: f32,
    pub pitch: f32,
}
