use amml_i18n::message::{Domain, Message};

/// A four-message domain that covers a plain message, a context, a plural and a note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleMessage {
    SignInAgain,
    Output,
    PlayersNearby,
    NeverTranslated,
    Greeting,
}

impl Message for SampleMessage {
    const DOMAIN: Domain = Domain::new("amml-ui");
    const ALL: &'static [Self] =
        &[Self::SignInAgain, Self::Output, Self::PlayersNearby, Self::NeverTranslated, Self::Greeting];

    fn index(self) -> usize {
        self as usize
    }

    fn msgid(self) -> &'static str {
        match self {
            Self::SignInAgain => "Sign In Again",
            Self::Output => "Output",
            Self::PlayersNearby => "{count} player nearby",
            Self::NeverTranslated => "Never Translated",
            Self::Greeting => "Hello, {name}",
        }
    }

    fn context(self) -> Option<&'static str> {
        match self {
            Self::Output => Some("audio"),
            _ => None,
        }
    }

    fn plural(self) -> Option<&'static str> {
        match self {
            Self::PlayersNearby => Some("{count} players nearby"),
            _ => None,
        }
    }

    fn note(self) -> Option<&'static str> {
        match self {
            Self::Output => Some("The audio device that plays sound"),
            _ => None,
        }
    }
}
