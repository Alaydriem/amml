use amml_i18n::text::Text;

use super::LibraryAppBarTab;

/// What the Library app bar shows. Every word comes from the host.
pub struct LibraryAppBarView {
    /// The letters on the brand tile.
    pub monogram: Text,
    /// The app name next to the tile.
    pub name: Text,
    pub library: Text,
    pub settings: Text,
    pub refresh: Text,
    pub open_file: Text,
    /// The last button; `None` while no replay is open to go back to.
    pub back: Option<Text>,
    pub tab: LibraryAppBarTab,
}
