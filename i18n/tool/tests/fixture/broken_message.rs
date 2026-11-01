use amml_i18n::message::{Domain, Message};

/// A message enum with one of each catalog problem: an empty msgid, a repeated key, plural placeholders that differ,
/// and an index that is not the position in `ALL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrokenMessage {
    Blank,
    Save,
    SaveAgain,
    Files,
    Misplaced,
}

impl Message for BrokenMessage {
    const DOMAIN: Domain = Domain::new("broken");
    const ALL: &'static [Self] = &[Self::Blank, Self::Save, Self::SaveAgain, Self::Files, Self::Misplaced];

    fn index(self) -> usize {
        match self {
            Self::Misplaced => 9,
            other => other as usize,
        }
    }

    fn msgid(self) -> &'static str {
        match self {
            Self::Blank => "  ",
            Self::Save | Self::SaveAgain => "Save",
            Self::Files => "{count} file",
            Self::Misplaced => "Misplaced",
        }
    }

    fn context(self) -> Option<&'static str> {
        None
    }

    fn plural(self) -> Option<&'static str> {
        match self {
            Self::Files => Some("{total} files"),
            _ => None,
        }
    }

    fn note(self) -> Option<&'static str> {
        None
    }
}
