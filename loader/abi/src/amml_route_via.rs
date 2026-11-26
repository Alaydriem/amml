/// How the game asked for a screen route: a push onto the router history or a replace of its top.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlRouteVia(pub u32);

impl AmmlRouteVia {
    pub const PUSH: Self = Self(1);
    pub const REPLACE: Self = Self(2);
}
