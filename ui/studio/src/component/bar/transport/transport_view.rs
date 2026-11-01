use amml_i18n::text::Text;

/// What the transport shows.
pub struct TransportView {
    pub playing: bool,
    /// The playhead time, as 00:00:32.95.
    pub timecode: Text,
    /// The replay length, shown after the timecode when given.
    pub duration: Option<Text>,
    /// The playback speed, as 1x.
    pub speed: Text,
    pub looping: bool,
    pub snap: bool,
    /// The composition guides are on.
    pub guides: bool,
    /// The timeline zoom, from 0 (all of the replay) to 1 (closest).
    pub zoom: f32,
}
