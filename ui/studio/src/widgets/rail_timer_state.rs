// a running capture (red REC) or a paused one (amber PAUSED)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RailTimerState {
    Live,
    Paused,
}
