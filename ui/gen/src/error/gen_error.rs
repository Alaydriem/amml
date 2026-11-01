use std::path::PathBuf;

/// Every way the generator can fail. Each token error names the token.
#[derive(Debug, thiserror::Error)]
pub enum GenError {
    #[error("token {token}: value `{raw}` is not a value this token type accepts")]
    BadValue { token: String, raw: String },
    #[error(
        "token {token}: the name does not fit its type, is used twice, or does not make a Rust name \
         (lowercase words of letters and digits joined by '-', starting with a letter, not a keyword)"
    )]
    BadName { token: String },
    #[error("token {token}: alias target {target} is not a token of the same type")]
    MissingAlias { token: String, target: String },
    #[error("token {token}: an amml-ui token must not alias the amml-studio-ui token {target}")]
    WrongDirectionAlias { token: String, target: String },
    #[error("token {token}: its aliases form a loop")]
    AliasCycle { token: String },
    #[error("tokens with no owner in ownership.json: {}", tokens.join(", "))]
    Unowned { tokens: Vec<String> },
    #[error("selector {selector}: {token} is not a colour token that this selector's crate can use")]
    BadSelector { selector: String, token: String },
    #[error("{message}")]
    Usage { message: String },
    #[error("{}: {source}", path.display())]
    Json { path: PathBuf, source: serde_json::Error },
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: std::io::Error },
}
