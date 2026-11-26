use std::fmt;

use crate::{ChordError, KeyCode, Modifiers};

/// A key with the modifiers held for it, as the settings text writes it: `Ctrl+Shift+M`, `F3+G`,
/// `Ctrl+LeftBracket`, `Alt+VK_0xE9`.
///
/// The text is the one modmanager settings use: the last part is the key, the parts before it are
/// modifiers (`Ctrl`, `Shift`, `Alt`, `F3`) in any order and any case, each at most once. F3 is only a
/// modifier, so a chord whose key is F3 does not parse. A modifier key alone (`Ctrl`) is a chord whose
/// key is that modifier. `Display` writes the canonical form: Ctrl, Shift, Alt, F3, then the key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chord {
    pub key: KeyCode,
    /// Compared with `Modifiers::generic`, so either side of a modifier matches. The Windows keys
    /// have no place in the text and are not compared.
    pub modifiers: Modifiers,
}

impl Chord {
    const CTRL: &'static str = "Ctrl";
    const SHIFT: &'static str = "Shift";
    const ALT: &'static str = "Alt";
    const F3: &'static str = "F3";

    /// Reads a chord from its settings text.
    pub fn parse(text: &str) -> Result<Chord, ChordError> {
        let (modifier_text, key_name) = match text.rsplit_once('+') {
            Some((modifiers, key_name)) => (Some(modifiers), key_name),
            None => (None, text),
        };
        let key = KeyCode::from_name(key_name).ok_or_else(|| ChordError::UnknownKey(key_name.to_string()))?;
        if key == KeyCode::F3 {
            return Err(ChordError::F3IsTheKey);
        }
        let mut modifiers = Modifiers::NONE;
        for part in modifier_text.into_iter().flat_map(|text| text.split('+')) {
            let modifier = Self::modifier(part).ok_or_else(|| ChordError::UnknownModifier(part.to_string()))?;
            if (modifiers.shift() && modifier.shift())
                || (modifiers.ctrl() && modifier.ctrl())
                || (modifiers.alt() && modifier.alt())
                || (modifiers.f3() && modifier.f3())
            {
                return Err(ChordError::RepeatedModifier(part.to_string()));
            }
            modifiers = modifiers | modifier;
        }
        Ok(Chord { key, modifiers })
    }

    fn modifier(part: &str) -> Option<Modifiers> {
        [
            (Self::CTRL, Modifiers::CTRL),
            (Self::SHIFT, Modifiers::SHIFT),
            (Self::ALT, Modifiers::ALT),
            (Self::F3, Modifiers::F3),
        ]
        .into_iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(part))
        .map(|(_, modifier)| modifier)
    }

    /// Whether a press of `key` with `modifiers` held is this chord. The chord's key matches itself,
    /// and a generic key (`Shift`) also matches both sides. The modifiers must be the chord's: either
    /// side counts, and the modifier that `key` itself is is not counted.
    pub fn matches(&self, key: KeyCode, modifiers: Modifiers) -> bool {
        if self.key != key && self.key != key.generic() {
            return false;
        }
        let own = key.generic();
        let same = |held: bool, wanted: bool, is_own: bool| is_own || held == wanted;
        same(modifiers.shift(), self.modifiers.shift(), own == KeyCode::SHIFT)
            && same(modifiers.ctrl(), self.modifiers.ctrl(), own == KeyCode::CONTROL)
            && same(modifiers.alt(), self.modifiers.alt(), own == KeyCode::MENU)
            && same(modifiers.f3(), self.modifiers.f3(), own == KeyCode::F3)
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (held, name) in [
            (self.modifiers.ctrl(), Self::CTRL),
            (self.modifiers.shift(), Self::SHIFT),
            (self.modifiers.alt(), Self::ALT),
            (self.modifiers.f3(), Self::F3),
        ] {
            if held {
                write!(formatter, "{name}+")?;
            }
        }
        match self.key.name() {
            Some(name) => formatter.write_str(name),
            None => write!(formatter, "{}{:X}", KeyCode::HEX_PREFIX, self.key.0),
        }
    }
}
