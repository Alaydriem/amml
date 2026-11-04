use super::{ExportFormat, ExportFrameRate, ExportH265Profile, ExportResolution};

/// One segment of the export dialog: a format, a size or a frame rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportChoice {
    Format(ExportFormat),
    Resolution(ExportResolution),
    FrameRate(ExportFrameRate),
    H265Profile(ExportH265Profile),
}
