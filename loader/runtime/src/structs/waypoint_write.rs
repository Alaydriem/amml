use crate::SavedWaypoint;

/// One write `WaypointTags::tag` asks the caller to make into the colour member of a waypoint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaypointWrite {
    /// The tag colour: four floats, and the colour then counts as set.
    Tag([f32; 4]),
    /// The saved bytes of the colour of a waypoint of a frame that was still open.
    Saved([u8; SavedWaypoint::COLOR_BYTES]),
}
