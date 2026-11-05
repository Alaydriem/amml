use crate::RegistryView;

// What a `RegistryRefresher` keeps between requests.
pub(crate) struct RefreshState {
    // counts the settings changes
    pub(crate) generation: u64,
    // the id of the last refresh that started
    pub(crate) last_id: u64,
    // the refresh in flight: its id and the generation it started at
    pub(crate) running: Option<(u64, u64)>,
    // the id of the last refresh that ended, with or without views
    pub(crate) done: u64,
    // the views of the last refresh that ended normally, with its id
    pub(crate) views: Option<(u64, Vec<RegistryView>)>,
}

impl RefreshState {
    pub(crate) fn new() -> RefreshState {
        RefreshState { generation: 0, last_id: 0, running: None, done: 0, views: None }
    }
}
