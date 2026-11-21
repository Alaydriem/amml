use std::collections::HashMap;

use crate::structs::ViewEntry;

/// What `ScreenBook` keeps: the next id, the id of each live controller, and each live view with its id and
/// controller.
pub(crate) struct ScreenBookState {
    pub(crate) next: u64,
    pub(crate) screens: HashMap<usize, u64>,
    pub(crate) views: HashMap<usize, ViewEntry>,
}
