use super::{AreaUse, EventUse};

/// One `feature` block of a mod's `uses` block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeatureUse {
    pub name: String,
    pub events: Vec<EventUse>,
    pub areas: Vec<AreaUse>,
}
