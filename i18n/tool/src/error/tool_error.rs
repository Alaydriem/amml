use std::path::PathBuf;

/// Everything the tool can refuse. Each message names the file or msgid at fault.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("{}: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{origin}: the catalog cannot be read: {message}")]
    Catalog { origin: String, message: String },
    #[error("plural expression {source_text:?}: {message}")]
    PluralExpression { source_text: String, message: String },
    #[error("{0}")]
    PluralForms(String),
    #[error("{locale}: the translation of {msgid:?} has placeholders {found:?}, but the English text has {expected:?}")]
    Placeholder { locale: String, msgid: String, expected: Vec<String>, found: Vec<String> },
    #[error("{} is out of date; run `mise run i18n:extract`", path.display())]
    StalePot { path: PathBuf },
    #[error("pack JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("usage: {0}")]
    Usage(String),
}
