//! Paper: Editor components, the viewer artboards (`2LV-0`, `30B-0`, `3I7-0`, `3TE-0`) and the path map (`46E-0`).
//! The game picture itself comes from the host; these parts are drawn over it.

mod composition_guides;
mod edit_lock_banner;
mod path_map;
mod viewer_chips;
mod viewer_overlay;
mod viewer_pill;

pub use composition_guides::CompositionGuides;
pub use edit_lock_banner::EditLockBanner;
pub use path_map::{PathMap, PathMapAction, PathMapCamera, PathMapPoint, PathMapView};
pub use viewer_chips::{
    CameraReadout, ReadoutLayout, ViewerChips, ViewerChipsView, ViewerOwner, ViewerSource, ViewerTone,
};
pub use viewer_overlay::ViewerOverlay;
pub(crate) use viewer_pill::ViewerPill;
