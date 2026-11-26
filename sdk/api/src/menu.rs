use amml_abi::AmmlMenu;

/// A menu page `Client::open_menu` can open.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    /// The Servers tab of the Play screen.
    Servers,
}

impl Menu {
    pub(crate) fn to_abi(self) -> AmmlMenu {
        match self {
            Self::Servers => AmmlMenu::SERVERS,
        }
    }
}
