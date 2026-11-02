use serde::{Deserialize, Serialize};

/// One search row, written as the array `[id, title, tags, apis, page]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameRow(pub String, pub String, pub Vec<String>, pub Vec<u32>, pub u32);
