/// Paper: `3TE-0` section C. How much of the camera readout fits the picture: all of it from 980 px, the position
/// and angles from 720 px, and below that the position only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadoutLayout {
    Wide,
    Middle,
    Small,
}

impl ReadoutLayout {
    // Paper: the picture widths where the readout drops its runs
    const MIDDLE: f32 = 720.0;
    const WIDE: f32 = 980.0;

    /// The layout for a picture `width` wide.
    pub fn for_width(width: f32) -> Self {
        if width >= Self::WIDE {
            Self::Wide
        } else if width >= Self::MIDDLE {
            Self::Middle
        } else {
            Self::Small
        }
    }
}
