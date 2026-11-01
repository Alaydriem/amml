/// A control of the transport that has a place on the bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportControl {
    ToSceneStart,
    PreviousKey,
    BackFive,
    Play,
    ForwardFive,
    NextKey,
    ToSceneEnd,
    Timecode,
    Speed,
    Loop,
    Snap,
    Guides,
    ZoomOut,
    ZoomSlider,
    ZoomIn,
    AddCameraKey,
    Fit,
}
