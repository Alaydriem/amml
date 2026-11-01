use std::fmt;

/// One fault in a message enum. `index` is the position in `Message::ALL`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogProblem {
    EmptyMsgid {
        index: usize,
    },
    /// The msgid and context are the same as an earlier message, so a translator sees one entry for both.
    RepeatedKey {
        index: usize,
        msgid: &'static str,
    },
    /// The singular and plural forms use different `{name}` placeholders.
    PlaceholderMismatch {
        index: usize,
        msgid: &'static str,
    },
    /// `Message::index` does not give the position in `Message::ALL`.
    IndexOutOfPlace {
        position: usize,
        index: usize,
    },
}

impl fmt::Display for CatalogProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMsgid { index } => write!(formatter, "message {index} has an empty msgid"),
            Self::RepeatedKey { index, msgid } => {
                write!(formatter, "message {index} ({msgid:?}) repeats the msgid and context of an earlier message")
            }
            Self::PlaceholderMismatch { index, msgid } => {
                write!(formatter, "message {index} ({msgid:?}) has other placeholders in its plural form")
            }
            Self::IndexOutOfPlace { position, index } => {
                write!(formatter, "message {position} gives index {index}, not its position in ALL")
            }
        }
    }
}
