use super::{AdapterShape, NeedKind};

/// The game shapes an API event or area can run on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdapterNeed {
    pub id: &'static str,
    pub kind: NeedKind,
    pub version: u32,
    pub shapes: &'static [AdapterShape],
}

impl AdapterNeed {
    /// An event need at version 1.
    pub const fn event(id: &'static str, shapes: &'static [AdapterShape]) -> Self {
        Self { id, kind: NeedKind::Event, version: 1, shapes }
    }

    /// An event need at `version`, for an event struct that grew.
    pub const fn event_at(id: &'static str, version: u32, shapes: &'static [AdapterShape]) -> Self {
        Self { id, kind: NeedKind::Event, version, shapes }
    }

    /// An area need at version 1.
    pub const fn area(id: &'static str, shapes: &'static [AdapterShape]) -> Self {
        Self { id, kind: NeedKind::Area, version: 1, shapes }
    }

    /// An area need at `version`, for an area whose calls or values grew.
    pub const fn area_at(id: &'static str, version: u32, shapes: &'static [AdapterShape]) -> Self {
        Self { id, kind: NeedKind::Area, version, shapes }
    }
}
