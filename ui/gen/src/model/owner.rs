use serde::Deserialize;

/// The crate that a token is generated into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub enum Owner {
    #[serde(rename = "amml-ui")]
    AmmlUi,
    #[serde(rename = "amml-studio-ui")]
    StudioUi,
}
