use std::collections::BTreeSet;

/// What a function holds that tends to stay the same between builds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Fingerprint {
    /// The string literals the function references.
    pub strings: BTreeSet<String>,
    /// The names of the imported functions it uses through the IAT.
    pub imports: BTreeSet<String>,
    /// Immediates of 0x10000 or more, negative immediates, and float or double literals.
    pub constants: BTreeSet<u64>,
    /// The number of call instructions.
    pub calls: u32,
    /// The bytes in every range of the function.
    pub size: u32,
}

impl Fingerprint {
    /// The number of strings, imports and constants.
    pub fn anchors(&self) -> usize {
        self.strings.len() + self.imports.len() + self.constants.len()
    }
}
