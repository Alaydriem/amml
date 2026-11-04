use amml_gdk::GameVersion;

// An instance the launcher does not manage yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdoptionCandidate {
    pub name: String,
    pub game: GameVersion,
    pub amml: Option<String>,
    pub mod_count: usize,
    pub has_version_in_store: bool,
}
