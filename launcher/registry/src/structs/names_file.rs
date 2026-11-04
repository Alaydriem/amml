use serde::{Deserialize, Serialize};

use super::NameRow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamesFile {
    pub schema: u32,
    pub rows: Vec<NameRow>,
}
