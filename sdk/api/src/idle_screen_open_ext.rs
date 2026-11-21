use amml_abi::IdleScreenOpen;

use crate::RouteVia;

/// Typed reads of an `IdleScreenOpen`.
pub trait IdleScreenOpenExt {
    /// Whether the game pushed or replaced the idle route.
    fn via(&self) -> RouteVia;
}

impl IdleScreenOpenExt for IdleScreenOpen {
    fn via(&self) -> RouteVia {
        RouteVia::from(self.via)
    }
}
