use egui::Pos2;

/// What a click on the scene band row asks for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneBandsAction {
    /// Make the scene at this index of the bands the active one.
    Select(usize),
    /// A right-click on a scene: its menu at the pointer.
    OpenMenu(usize, Pos2),
    /// The "+" in the header cell.
    Add,
}
