use std::ops::BitOr;

use amml_abi::AmmlModifiers;

/// Held modifier keys: the loader's `AmmlModifiers` bits, plus F3 held as a modifier.
///
/// The loader reports Shift, Ctrl, Alt and Windows, by side when the build gives the side and as the
/// generic key when it does not. F3 is not an OS modifier: `Input::modifiers` sets it while F3 is
/// down, and the modifiers of a `KeyInput` never carry it (set it with `with_f3`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Modifiers {
    bits: u16,
    f3: bool,
}

impl Modifiers {
    pub const NONE: Self = Self { bits: 0, f3: false };
    pub const SHIFT: Self = Self::from_bits(AmmlModifiers::SHIFT);
    pub const CTRL: Self = Self::from_bits(AmmlModifiers::CTRL);
    pub const ALT: Self = Self::from_bits(AmmlModifiers::ALT);
    pub const LSHIFT: Self = Self::from_bits(AmmlModifiers::LSHIFT);
    pub const RSHIFT: Self = Self::from_bits(AmmlModifiers::RSHIFT);
    pub const LCTRL: Self = Self::from_bits(AmmlModifiers::LCTRL);
    pub const RCTRL: Self = Self::from_bits(AmmlModifiers::RCTRL);
    pub const LALT: Self = Self::from_bits(AmmlModifiers::LALT);
    pub const RALT: Self = Self::from_bits(AmmlModifiers::RALT);
    pub const LWIN: Self = Self::from_bits(AmmlModifiers::LWIN);
    pub const RWIN: Self = Self::from_bits(AmmlModifiers::RWIN);
    /// F3 held as a modifier.
    pub const F3: Self = Self { bits: 0, f3: true };

    const fn from_bits(raw: AmmlModifiers) -> Self {
        Self { bits: raw.0, f3: false }
    }

    const fn any(self, mask: u16) -> bool {
        self.bits & mask != 0
    }

    /// Either Shift, or the generic Shift.
    pub fn shift(self) -> bool {
        self.any(AmmlModifiers::LSHIFT.0 | AmmlModifiers::RSHIFT.0 | AmmlModifiers::SHIFT.0)
    }

    /// Either Ctrl, or the generic Ctrl.
    pub fn ctrl(self) -> bool {
        self.any(AmmlModifiers::LCTRL.0 | AmmlModifiers::RCTRL.0 | AmmlModifiers::CTRL.0)
    }

    /// Either Alt, or the generic Alt.
    pub fn alt(self) -> bool {
        self.any(AmmlModifiers::LALT.0 | AmmlModifiers::RALT.0 | AmmlModifiers::ALT.0)
    }

    /// Either Windows key.
    pub fn win(self) -> bool {
        self.any(AmmlModifiers::LWIN.0 | AmmlModifiers::RWIN.0)
    }

    /// F3 held as a modifier.
    pub fn f3(self) -> bool {
        self.f3
    }

    /// The loader's bits, without F3.
    pub fn raw(self) -> AmmlModifiers {
        AmmlModifiers(self.bits)
    }

    /// These modifiers with F3 held or not.
    pub fn with_f3(self, held: bool) -> Self {
        Self { f3: held, ..self }
    }

    /// The same modifiers with the sides folded: either Shift, Ctrl or Alt gives the generic bit, and
    /// either Windows key gives `LWIN`.
    pub fn generic(self) -> Self {
        let mut generic = Self { bits: 0, f3: self.f3 };
        for (held, bit) in [
            (self.shift(), AmmlModifiers::SHIFT),
            (self.ctrl(), AmmlModifiers::CTRL),
            (self.alt(), AmmlModifiers::ALT),
            (self.win(), AmmlModifiers::LWIN),
        ] {
            if held {
                generic.bits |= bit.0;
            }
        }
        generic
    }
}

impl BitOr for Modifiers {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self { bits: self.bits | other.bits, f3: self.f3 || other.f3 }
    }
}

impl From<AmmlModifiers> for Modifiers {
    fn from(raw: AmmlModifiers) -> Self {
        Self::from_bits(raw)
    }
}
