/// The kind of a drawn shape. The game makes the shape object of a kind once, at the shape's first payload; a later
/// payload of the same game id updates that object and cannot change its kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShapeKind {
    Line,
    Box,
    Arrow,
}
