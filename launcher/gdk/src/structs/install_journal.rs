use super::Channel;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Written before the first registration change and deleted after the restore. Its presence on disk means a
// change may be half done and the recorded package must be put back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallJournal {
    pub original_full_name: String,
    pub original_package: PathBuf,
    pub wanted_full_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staging: Option<PathBuf>,
    pub channel: Channel,
}
