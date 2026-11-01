use crate::widget::{ColourValue, GridCellRef};

/// The values the gallery's live widgets change. Each sheet adds the fields it needs.
#[derive(Debug)]
pub struct GalleryState {
    pub inspector_tab: usize,
    pub library_tab: usize,
    /// The "Enabled" toggle on the Page header sheet.
    pub mod_enabled: bool,
    /// The Section card sheet's settings.
    pub show_map: bool,
    pub zoom: f64,
    pub scan_radius: f64,
    pub shape: Option<usize>,
    pub chunk_grid: bool,
    pub grid_colour: ColourValue,
    /// The Grid placer sheet's three pieces: two placed, one in the tray.
    pub grid_pieces: [Option<GridCellRef>; 3],
}

impl Default for GalleryState {
    // the selections Paper shows
    fn default() -> Self {
        Self {
            inspector_tab: 1,
            library_tab: 0,
            mod_enabled: true,
            show_map: true,
            zoom: 3.0,
            scan_radius: 12.0,
            shape: Some(0),
            chunk_grid: false,
            grid_colour: ColourValue { rgba: [0xFF, 0xFF, 0xFF, 0x40], alpha: true },
            grid_pieces: [Some(GridCellRef::new(0, 0)), Some(GridCellRef::new(1, 0)), None],
        }
    }
}
