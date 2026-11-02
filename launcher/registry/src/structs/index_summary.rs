use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::ApiSummary;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexSummary {
    pub page_size: u32,
    // keyed by API version as text
    pub apis: BTreeMap<String, ApiSummary>,
}
