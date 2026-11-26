/// What an adapter need offers to mods: an event to listen to, or an area of calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeedKind {
    Event,
    Area,
}
