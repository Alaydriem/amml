use amml_data::NeedKind;

/// Whether one event or area a mod uses can run on a build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompatRow {
    pub mod_id: String,
    pub feature: String,
    pub kind: NeedKind,
    pub name: String,
    pub required: bool,
    pub available: bool,
    /// The game shape that holds, when the event or area is available.
    pub shape: Option<String>,
    /// The unresolved entries, or the reason the catalog has no such event or area.
    pub missing: Vec<String>,
    /// The loader this check is built with does not serve the row's version (or does not know the id): a newer
    /// loader is needed, whatever the build holds.
    pub loader_too_old: bool,
}
