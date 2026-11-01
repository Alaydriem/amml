/// A message catalog: one .pot file and one set of packs.
///
/// Each mod declares its own domains, so amml-i18n keeps no list of mods:
///
/// ```
/// use amml_i18n::message::Domain;
///
/// const FREECAM: Domain = Domain::new("freecam");
/// assert_eq!(FREECAM.name(), "freecam");
/// ```
///
/// Each domain has exactly one `Message` enum in a process. An enum that is split by area gets
/// its own domain. Two mods may use the same name, because each mod loads its own packs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Domain {
    name: &'static str,
}

impl Domain {
    /// A domain named `name`. The name is the .pot file stem and the folder name of the packs,
    /// so it may hold only lowercase ASCII letters, digits and `-`, and must not be empty.
    ///
    /// # Panics
    ///
    /// Panics on any other name. In a `const` the panic is a compile error.
    pub const fn new(name: &'static str) -> Self {
        let bytes = name.as_bytes();
        assert!(!bytes.is_empty(), "domain name must not be empty");
        let mut at = 0;
        while at < bytes.len() {
            let byte = bytes[at];
            assert!(
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-',
                "domain name may hold only lowercase ASCII letters, digits and '-'"
            );
            at += 1;
        }
        Self { name }
    }

    /// The catalog name: the .pot file stem and the folder name of its packs.
    pub const fn name(self) -> &'static str {
        self.name
    }
}
