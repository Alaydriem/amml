use serde::Serialize;
use ts_rs::TS;

// The AMML release a mod release needs, and the newest game that AMML release lists (when the official registry
// lists it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NeedsAmmlView {
    pub version: String,
    pub game: Option<String>,
}

impl From<&amml_launcher_core::NeedsAmml> for NeedsAmmlView {
    fn from(needs: &amml_launcher_core::NeedsAmml) -> NeedsAmmlView {
        NeedsAmmlView { version: needs.version.clone(), game: needs.game.clone() }
    }
}
