use crate::tokens::selector::StatusColor;

/// Paper: `Coverage panel` (`9GC-0`, `9H2-0`). Which rows the panel lists: all of them, or one status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageFilter {
    All,
    Status(StatusColor),
}

impl CoverageFilter {
    /// The filter pills in Paper order.
    pub const ALL: [Self; 6] = [
        Self::All,
        Self::Status(StatusColor::Works),
        Self::Status(StatusColor::Engine),
        Self::Status(StatusColor::Planned),
        Self::Status(StatusColor::Mockup),
        Self::Status(StatusColor::Missing),
    ];

    pub fn shows(self, status: StatusColor) -> bool {
        match self {
            Self::All => true,
            Self::Status(only) => only == status,
        }
    }
}
