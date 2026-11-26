use amml_abi::EventCallbackFn;

/// One listener the fake loader holds: the callback and the `user` pointer, as an address.
#[derive(Clone, Copy)]
pub(crate) struct FakeListener {
    pub(crate) callback: EventCallbackFn,
    pub(crate) user: usize,
}
