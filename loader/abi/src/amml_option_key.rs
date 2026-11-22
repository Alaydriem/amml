/// An integer game option that `AmmlOptionsTable::get_int` and `set_int` reach.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid. Zero is not a key.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AmmlOptionKey(pub u32);

impl AmmlOptionKey {
    /// The vertical sync option (`gfx_vsync`): 0 off, 1 on, 2 adaptive.
    pub const VSYNC: Self = Self(1);
    /// The effective maximum frame rate option of the game (the one `getEffectiveMaxFramerateOption` gives); 0 is no
    /// limit.
    pub const MAX_FRAMERATE: Self = Self(2);
}
