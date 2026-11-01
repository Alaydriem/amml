use std::borrow::Cow;

use crate::localize::Localizer;

use super::{MessageRef, UserText};

/// The value of one named placeholder.
#[derive(Debug, Clone)]
pub enum ArgValue {
    Int(i64),
    UInt(u64),
    /// A number and how many decimals to show.
    Float(f64, u8),
    Text(UserText),
    Message(MessageRef),
}

impl ArgValue {
    pub fn render<'a>(&'a self, localizer: &'a Localizer) -> Cow<'a, str> {
        match self {
            Self::Int(value) => Cow::Owned(value.to_string()),
            Self::UInt(value) => Cow::Owned(value.to_string()),
            Self::Float(value, decimals) => Cow::Owned(format!("{value:.*}", usize::from(*decimals))),
            Self::Text(text) => Cow::Borrowed(text.as_str()),
            Self::Message(message) => Cow::Borrowed(message.text(localizer)),
        }
    }
}
