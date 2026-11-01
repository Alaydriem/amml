//! Paper: Menus + Dialogs, the inspector frame (`Key active` `4JQ-0`, `Locked` `4MU-0`) and the undo history
//! (`46V-0`), with the rail and drawer of the small window (`layout-brief.md` item 3). The tab contents are the
//! host's (Plan 08).

mod inspector_drawer;
mod inspector_locked;
mod inspector_rail;
mod inspector_tab;
mod inspector_tabs;
mod key_header;
mod player_row;
mod undo_history_popover;

pub use inspector_drawer::InspectorDrawer;
pub use inspector_locked::InspectorLocked;
pub use inspector_rail::InspectorRail;
pub use inspector_tab::InspectorTab;
pub use inspector_tabs::InspectorTabs;
pub use key_header::{KeyHeader, KeyHeaderTone, KeyHeaderView};
pub use player_row::{PlayerRow, PlayerRowAction, PlayerRowView};
pub use undo_history_popover::{UndoHistoryAction, UndoHistoryPopover, UndoHistoryView};
