// The AMML release a mod release needs, and the newest game that AMML release lists (when the root lists it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeedsAmml {
    pub version: String,
    pub game: Option<String>,
}
