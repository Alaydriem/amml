use egui::Pos2;

/// A click in a replay row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplayRowAction {
    /// One click: this replay is the selected one.
    Select,
    /// Two quick clicks: open this replay.
    Open,
    /// A right click at this point: the host opens its menu for this replay.
    Menu(Pos2),
}
