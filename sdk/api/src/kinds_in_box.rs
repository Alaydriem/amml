/// What `BlockSource::kinds_in_box` found in a box.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KindsInBox {
    /// The full names (such as `minecraft:stone`) of the distinct block types of the box, in the order the loader met
    /// them: x, then z, then y, each upward. Air (`minecraft:air`) is a kind like any other.
    Kinds(Vec<String>),
    /// A chunk of the box is not loaded, or a block could not be read. Nothing was listed; ask again later.
    NotLoaded,
    /// The box holds more kinds than `max_kinds`. Nothing was listed.
    TooMany,
}
