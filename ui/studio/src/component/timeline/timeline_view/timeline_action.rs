use egui::Pos2;

use super::MenuTarget;

/// What the timeline asks the host to do. Lanes and keys are indexes into `TimelineViewModel::tracks` and each
/// lane's keys; scenes are indexes into its scenes.
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineAction {
    /// The playhead is being dragged on the ruler; sent each frame, in seconds clamped to the replay.
    Scrub(f64),
    Seek(f64),
    /// Show the replay from this time at the left edge (the scroll bar was dragged).
    Pan(f64),
    /// Scroll the lanes to this offset.
    Scroll(f32),
    OpenMenu(MenuTarget, Pos2),
    SelectScene(usize),
    AddScene,
    SelectTrack(usize),
    ToggleHidden(usize),
    ToggleLocked(usize),
    AddTrack,
    SelectKey {
        lane: usize,
        key: usize,
    },
    /// The keys inside the box being dragged, sent each frame of the drag.
    SelectKeys(Vec<(usize, usize)>),
    MoveKey {
        lane: usize,
        key: usize,
        seconds: f64,
    },
    /// The key drag ended; the host records one undo step for the whole move.
    MoveKeyDone {
        lane: usize,
        key: usize,
    },
}
