/// The kind of a client connection of the packet wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WireKind {
    /// The client's connection to a world on this machine.
    Local,
    /// A connection to another machine.
    Remote,
}
