use super::GridCellRef;

/// What a grid placer reported this frame. The caller changes its model; the placer keeps none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GridPlacerResponse {
    /// Nothing changed. A drop on a taken cell is also `None`.
    #[default]
    None,
    /// The piece `key` was dropped or placed on the empty `cell`, from the tray or from another cell.
    Placed { key: u64, cell: GridCellRef },
    /// The piece `key` went back to the tray.
    Unplaced { key: u64 },
}
