/// Paper: Assets, `Brand / App icon`. The four exported sizes; the 16 px icon drops the echo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppIconSize {
    Large,
    Medium,
    Small,
    Tiny,
}

impl AppIconSize {
    pub const ALL: [AppIconSize; 4] = [Self::Large, Self::Medium, Self::Small, Self::Tiny];

    pub fn pixels(self) -> u32 {
        match self {
            Self::Large => 128,
            Self::Medium => 48,
            Self::Small => 32,
            Self::Tiny => 16,
        }
    }
}
