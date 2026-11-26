use crate::ShapeKind;

/// One shape change a mod asks for: draw (or move) its shape `id` as `kind`, or remove it when `kind` is `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeRequest {
    pub id: u64,
    pub kind: Option<ShapeKind>,
}
