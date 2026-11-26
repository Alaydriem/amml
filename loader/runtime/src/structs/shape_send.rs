use crate::ShapeKind;

/// One payload the loader hands to the game: the shape `network_id` drawn as `kind`, or removed when `kind` is `None`.
/// `op` is the index of the request it comes from, whose geometry the loader writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeSend {
    pub op: usize,
    pub network_id: u64,
    pub kind: Option<ShapeKind>,
}
