use egui::Pos2;

/// What a click on the ruler asks for, at a time in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RulerAction {
    /// The playhead is being dragged; sent each frame of the drag. The host shows the time but does not seek.
    Scrub(f64),
    /// The drag ended, or the ruler was clicked: seek once.
    Seek(f64),
    /// A right-click: the ruler menu ("New scene here") at the pointer.
    OpenMenu(f64, Pos2),
}
