use egui::Pos2;

/// A click in the scene switcher list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneSwitcherListAction {
    SelectOriginal,
    SelectScene(usize),
    /// A right click on a scene: the host opens the scene menu at the point.
    OpenMenu(usize, Pos2),
    /// The footer action at this index.
    Footer(usize),
}
