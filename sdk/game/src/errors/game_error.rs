use amml_sdk::SdkError;

/// Why the game entries did not resolve.
#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("required entry `{id}` did not resolve")]
    MissingEntry { id: String },
    #[error("no single complete entry set for the {group}: {detail}")]
    EntrySet { group: &'static str, detail: String },
    #[error("entry `{id}` failed to resolve: {source}")]
    Resolve {
        id: String,
        #[source]
        source: SdkError,
    },
}
