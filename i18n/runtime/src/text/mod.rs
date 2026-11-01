pub mod arg_value;
pub mod args;
pub mod message_ref;
pub mod placeholder_format;
pub mod user_text;

use std::borrow::Cow;

pub use arg_value::ArgValue;
pub use args::Args;
pub use message_ref::MessageRef;
pub use placeholder_format::PlaceholderFormat;
pub use user_text::UserText;

use crate::localize::Localizer;
use crate::message::Message;

/// Text a widget shows: a message, a formatted message, a plural, or a value from the model.
///
/// Widgets take `Text`, never `&str`, so every string on screen is either translatable or
/// marked as user data.
#[derive(Debug, Clone)]
pub enum Text {
    Message(MessageRef),
    Format(MessageRef, Args),
    Plural(MessageRef, u64, Args),
    User(UserText),
}

impl Text {
    pub fn message(message: impl Message) -> Self {
        Self::Message(MessageRef::of(message))
    }

    pub fn format(message: impl Message, args: Args) -> Self {
        Self::Format(MessageRef::of(message), args)
    }

    /// A plural message. `{count}` is filled with `count` unless `args` already names it.
    pub fn plural(message: impl Message, count: u64, args: Args) -> Self {
        let args = if args.contains("count") { args } else { args.uint("count", count) };
        Self::Plural(MessageRef::of(message), count, args)
    }

    pub fn user(text: UserText) -> Self {
        Self::User(text)
    }

    /// The text in the localizer's locale. Borrowed unless placeholders had to be filled.
    pub fn resolve<'a>(&'a self, localizer: &'a Localizer) -> Cow<'a, str> {
        match self {
            Self::Message(message) => Cow::Borrowed(message.text(localizer)),
            Self::Format(message, args) => {
                Cow::Owned(PlaceholderFormat::apply(message.text(localizer), args, localizer))
            }
            Self::Plural(message, count, args) => {
                Cow::Owned(PlaceholderFormat::apply(message.plural(localizer, *count), args, localizer))
            }
            Self::User(text) => Cow::Borrowed(text.as_str()),
        }
    }
}
