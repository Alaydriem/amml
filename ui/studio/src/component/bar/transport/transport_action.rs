/// What a click in the transport asks for. There is no timelapse toggle (spec 6.7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportAction {
    ToSceneStart,
    PreviousKey,
    BackFive,
    TogglePlay,
    ForwardFive,
    NextKey,
    ToSceneEnd,
    CycleSpeed,
    ToggleLoop,
    ToggleSnap,
    ToggleGuides,
    ZoomOut,
    ZoomIn,
    SetZoom(f32),
    AddCameraKey,
    Fit,
}
