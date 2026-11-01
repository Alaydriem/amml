use super::AppIconSize;

/// Paper: Assets, `Brand / App icon`, exported as PNG at 1x. The DV mark with an echo behind it.
pub struct AppIcon;

impl AppIcon {
    const LARGE: &'static [u8] = include_bytes!("../../assets/brand/app-icon-128.png");
    const MEDIUM: &'static [u8] = include_bytes!("../../assets/brand/app-icon-48.png");
    const SMALL: &'static [u8] = include_bytes!("../../assets/brand/app-icon-32.png");
    const TINY: &'static [u8] = include_bytes!("../../assets/brand/app-icon-16.png");

    /// The PNG file bytes at `size`.
    pub fn png(size: AppIconSize) -> &'static [u8] {
        match size {
            AppIconSize::Large => Self::LARGE,
            AppIconSize::Medium => Self::MEDIUM,
            AppIconSize::Small => Self::SMALL,
            AppIconSize::Tiny => Self::TINY,
        }
    }
}
