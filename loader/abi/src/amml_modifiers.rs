/// Held modifier keys, as bits. SHIFT, CTRL and ALT are set when the build gives only the generic key.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlModifiers(pub u16);

impl AmmlModifiers {
    pub const LSHIFT: Self = Self(0x1);
    pub const RSHIFT: Self = Self(0x2);
    pub const LCTRL: Self = Self(0x4);
    pub const RCTRL: Self = Self(0x8);
    pub const LALT: Self = Self(0x10);
    pub const RALT: Self = Self(0x20);
    pub const LWIN: Self = Self(0x40);
    pub const RWIN: Self = Self(0x80);
    pub const SHIFT: Self = Self(0x100);
    pub const CTRL: Self = Self(0x200);
    pub const ALT: Self = Self(0x400);
}
