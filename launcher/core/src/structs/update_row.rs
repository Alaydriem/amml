use crate::{NeedsAmml, UpdateStatus};

// One line of an update check. The AMML row has the id `alaydriem/amml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateRow {
    pub id: String,
    pub installed: Option<String>,
    pub available: Option<String>,
    pub status: UpdateStatus,
    // the newer AMML is for another API than the installed one, so mods need new releases
    pub api_change: bool,
    // set when the newest release needs a newer AMML than the instance has
    pub needs_amml: Option<NeedsAmml>,
}
