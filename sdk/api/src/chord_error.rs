/// Why a chord text does not parse.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ChordError {
    #[error("unknown key name: {0}")]
    UnknownKey(String),
    #[error("unknown modifier: {0}")]
    UnknownModifier(String),
    #[error("a modifier is written twice: {0}")]
    RepeatedModifier(String),
    #[error("F3 is only a modifier, never the key of a chord")]
    F3IsTheKey,
}
