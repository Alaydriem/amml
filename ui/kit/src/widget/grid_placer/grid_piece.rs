use amml_i18n::text::Text;

use super::GridCellRef;

/// One piece the caller gave a grid placer.
pub(crate) struct GridPiece {
    pub key: u64,
    pub label: Text,
    pub cell: Option<GridCellRef>,
}
