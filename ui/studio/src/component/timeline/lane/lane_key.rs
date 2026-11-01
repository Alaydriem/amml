use super::super::KeyKind;

/// A key on a lane: its time in seconds, its kind and whether it is selected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaneKey {
    pub seconds: f64,
    pub kind: KeyKind,
    pub selected: bool,
}
