/// The handle `EventBus::add` gives for one listener.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ListenerId(pub u64);
