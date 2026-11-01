//! The live widget sheets: one page per Paper widget sheet, drawn with the real widgets. The gallery
//! example and the snapshot tests both show these pages.

mod gallery_id;
mod gallery_sheet;
mod gallery_state;
mod open_row;
mod sample;
mod sample_colours;
mod sample_diamond;
pub mod sheet;
mod sheet_grid;

pub use gallery_id::GalleryId;
pub use gallery_sheet::GallerySheet;
pub use gallery_state::GalleryState;
pub use open_row::OpenRow;
pub use sample::Sample;
pub use sample_colours::SampleColours;
pub use sample_diamond::SampleDiamond;
pub use sheet_grid::SheetGrid;
