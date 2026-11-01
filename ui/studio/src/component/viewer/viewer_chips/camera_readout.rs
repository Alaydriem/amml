use amml_i18n::text::Text;

/// Paper: `3TE-0` section C. The camera position and angles, each already written as the caller wants it shown,
/// and an optional trailing note (as in the time and weather).
#[derive(Debug, Clone)]
pub struct CameraReadout {
    pub x: Text,
    pub y: Text,
    pub z: Text,
    pub yaw: Text,
    pub pitch: Text,
    pub roll: Text,
    pub extra: Option<Text>,
}
