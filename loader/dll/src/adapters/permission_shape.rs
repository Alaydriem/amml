use amml_data::AdapterCatalog;

/// How the loader reads the player permission level on this build: the catalog shape of `actor.permission` that
/// resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermissionShape {
    /// `Player::getPlayerPermissionLevel`: the level in `al` (26.63).
    Getter,
    /// `Player::getAbilitiesComponent`, then the byte at `AbilitiesComponent.mPlayerPermissions` (70.x).
    Abilities,
}

impl PermissionShape {
    /// The shape that goes with the catalog shape name `name`; `None` for a name the catalog does not give.
    pub fn from_catalog(name: &str) -> Option<Self> {
        match name {
            AdapterCatalog::PERMISSION_GETTER_SHAPE => Some(Self::Getter),
            AdapterCatalog::PERMISSION_ABILITIES_SHAPE => Some(Self::Abilities),
            _ => None,
        }
    }

    /// The catalog shape name, for the log line.
    pub fn name(self) -> &'static str {
        match self {
            Self::Getter => AdapterCatalog::PERMISSION_GETTER_SHAPE,
            Self::Abilities => AdapterCatalog::PERMISSION_ABILITIES_SHAPE,
        }
    }
}
