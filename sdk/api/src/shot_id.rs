/// The id of one screenshot, from `Overlay::screenshot`. `ScreenshotTaken.id` and
/// `ScreenshotWritten.id` carry it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShotId(pub u64);
