use amml_abi::AmmlRotation;

/// A view direction in degrees. `yaw` 0 faces +Z (south) and grows clockwise seen from above (90 faces -X, west);
/// `pitch` 0 is level and grows looking down (90 straight down, -90 straight up). The turn of `InputLease::turn`
/// (`Look`) uses the same signs: a turn of `Look { yaw: 10.0, pitch: 0.0 }` adds 10 to `yaw`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

impl From<AmmlRotation> for Rotation {
    fn from(value: AmmlRotation) -> Self {
        Self { yaw: value.yaw, pitch: value.pitch }
    }
}
