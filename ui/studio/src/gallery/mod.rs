//! The live component pages: one page per Paper artboard group, drawn with the real components.
//! The gallery example and the snapshot tests both show these pages.

mod gallery_page;
mod gallery_state;
mod page;

pub use gallery_page::GalleryPage;
pub use gallery_state::GalleryState;
pub use page::{
    BarRows, CaptureRailPage, CurvePlotPage, DialogsPage, EasingPopoverPage, EditLockBannerPage, GlyphsPage,
    HeroCardPage, InspectorPage, KeyHeaderPage, LibraryAppBarPage, PathMapPage, PlayerRowPage, ReplayRowPage,
    SceneCardPage, SceneSwitcherChipPage, SceneSwitcherListPage, SourceListPage, StatusBadgePage, StatusBarPage,
    TimelinePage, TimelineRowsPage, TimelineSamples, TimelineTracksPage, TopBarPage, TransportPage, ViewerChipsPage,
    ViewerPage, ViewerSamples,
};
