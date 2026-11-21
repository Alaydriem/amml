use amml_abi::EventCallbackFn;

/// One listener a mod registered through `AmmlEventsTable::listen` (with a non-null callback): the mod index that owns it, its
/// callback and the `user` pointer the mod passed, kept as an address.
#[derive(Clone, Copy, Debug)]
pub struct ModListener {
    pub owner: usize,
    pub callback: EventCallbackFn,
    pub user: usize,
}
