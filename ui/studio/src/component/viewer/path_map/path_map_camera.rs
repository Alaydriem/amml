use egui::Pos2;

/// The camera on the path map: its world position seen from above, and its heading in radians, clockwise from north.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathMapCamera {
    pub at: Pos2,
    pub heading: f32,
}
