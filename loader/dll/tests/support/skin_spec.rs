/// What a stand-in skin object holds: the main image, the animated images, the flags and the texts.
#[derive(Clone, Debug)]
pub struct SkinSpec {
    /// Width and height of the main image.
    pub size: (u32, u32),
    /// The blob size of the main image; `None` gives `width * height * 4`.
    pub blob_len: Option<usize>,
    /// Kind, width and height of each animated image.
    pub animated: Vec<(u32, u32, u32)>,
    pub persona: bool,
    pub premium: bool,
    pub geometry_name: Vec<u8>,
    pub resource_patch: Vec<u8>,
}

impl Default for SkinSpec {
    fn default() -> Self {
        Self {
            size: (2, 2),
            blob_len: None,
            animated: Vec::new(),
            persona: false,
            premium: false,
            geometry_name: b"geometry.humanoid.custom".to_vec(),
            resource_patch: b"{\"geometry\":{\"default\":\"geometry.humanoid.custom\"}}".to_vec(),
        }
    }
}
