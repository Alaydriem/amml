use serde::Serialize;
use ts_rs::TS;

// A newer launcher the updater found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LauncherUpdateView {
    pub version: String,
    // true when this install can update itself; otherwise the player gets the release page
    pub installs: bool,
    pub page: String,
}

impl LauncherUpdateView {
    pub fn new(version: &str, installs: bool) -> LauncherUpdateView {
        LauncherUpdateView {
            version: version.to_string(),
            installs,
            page: format!("https://github.com/alaydriem/amml/releases/tag/{version}"),
        }
    }
}
