use std::collections::HashMap;

use crate::OptionKey;

/// The options a `FakeGame` serves after `set_options`: what `Options` reads give. A key that is not in `ints` is an
/// option the loader cannot find: its reads give `None` and its writes `Err(NotAvailable)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FakeOptions {
    /// What `Options::view_distance_chunks` gives.
    pub view_distance: u32,
    /// The value of each integer option.
    pub ints: HashMap<OptionKey, i32>,
}

impl Default for FakeOptions {
    /// A view distance of 12 chunks and no integer option.
    fn default() -> Self {
        Self { view_distance: 12, ints: HashMap::new() }
    }
}
