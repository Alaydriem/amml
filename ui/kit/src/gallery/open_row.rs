use crate::state::InteractionState;
use crate::widget::MenuItem;

/// One row of a menu the gallery paints open: an item in a state, or a separator.
// a handful of rows per page, built once per frame; boxing the item buys nothing
#[allow(clippy::large_enum_variant)]
pub enum OpenRow {
    Item(MenuItem, InteractionState),
    Separator,
}
