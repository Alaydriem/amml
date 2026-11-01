use egui::Pos2;

/// A camera key on the path map, at its world position seen from above (x east, y south).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathMapPoint {
    pub at: Pos2,
    pub selected: bool,
}
