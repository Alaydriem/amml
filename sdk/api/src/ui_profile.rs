use amml_abi::AmmlUiProfile;

/// The UI profile option of the game: the layout family of its screens.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiProfile {
    Classic,
    Pocket,
}

impl From<AmmlUiProfile> for UiProfile {
    fn from(profile: AmmlUiProfile) -> Self {
        match profile {
            AmmlUiProfile::POCKET => Self::Pocket,
            _ => Self::Classic,
        }
    }
}

impl From<UiProfile> for AmmlUiProfile {
    fn from(profile: UiProfile) -> Self {
        match profile {
            UiProfile::Classic => Self::CLASSIC,
            UiProfile::Pocket => Self::POCKET,
        }
    }
}
