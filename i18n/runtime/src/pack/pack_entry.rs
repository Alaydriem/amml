use serde::{Deserialize, Serialize};

/// One translated message: a single string, or one string per plural form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackEntry {
    One(String),
    Many(Vec<String>),
}
