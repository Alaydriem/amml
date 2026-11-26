use std::collections::HashMap;

/// What `TopScreenWatch` keeps between ticks.
pub(crate) struct WatchState {
    // The screens drawn since the previous tick, each once, in the order of its last draw.
    pub(crate) drawn: Vec<u64>,
    // The screens drawn in the tick that ended last, in the same order.
    pub(crate) last_order: Vec<u64>,
    // The root control name of each live screen that was read.
    pub(crate) names: HashMap<u64, String>,
    // The top screen name of the previous tick when no drawn view matched it.
    pub(crate) unmatched: Option<String>,
    // The top screen the last change reported; 0 before the first change.
    pub(crate) kept: u64,
}
