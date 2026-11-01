use super::LibraryAppBarTab;

/// A click in the Library app bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryAppBarAction {
    /// The other tab was picked.
    SelectTab(LibraryAppBarTab),
    Refresh,
    OpenFile,
    /// Back to the replay that was open.
    Back,
}
