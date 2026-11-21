use amml_abi::KeyInput;

use crate::{KeyCode, Modifiers};

/// Typed reads of a `KeyInput`.
pub trait KeyInputExt {
    /// The key: the side code when the build gives the side, else the generic code.
    fn key_code(&self) -> KeyCode;
    /// The modifiers the user held, from the real key feed; never F3.
    fn modifiers(&self) -> Modifiers;
}

impl KeyInputExt for KeyInput {
    fn key_code(&self) -> KeyCode {
        KeyCode(self.key)
    }

    fn modifiers(&self) -> Modifiers {
        Modifiers::from(self.modifiers)
    }
}
