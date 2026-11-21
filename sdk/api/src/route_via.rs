use amml_abi::AmmlRouteVia;

/// How the game asked for a screen route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteVia {
    /// A push onto the router history.
    Push,
    /// A replace of the top of the router history.
    Replace,
    /// A value this crate does not know.
    Other(u32),
}

impl From<AmmlRouteVia> for RouteVia {
    fn from(via: AmmlRouteVia) -> Self {
        match via {
            AmmlRouteVia::PUSH => Self::Push,
            AmmlRouteVia::REPLACE => Self::Replace,
            AmmlRouteVia(other) => Self::Other(other),
        }
    }
}
