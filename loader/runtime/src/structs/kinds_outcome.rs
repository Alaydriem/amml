/// What a scan of the block kinds of a box found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KindsOutcome {
    /// The distinct block type ids of the box, in the order the scan met them.
    Kinds(Vec<u32>),
    /// A chunk of the box is not loaded, or a cell could not be read; no list is given.
    NotLoaded,
    /// The box holds more kinds than the scan was allowed; no list is given.
    TooMany,
}
