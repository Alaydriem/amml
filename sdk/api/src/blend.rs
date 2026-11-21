/// How a world mesh mixes with what is behind it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Blend {
    /// The mesh colour replaces what is behind.
    Opaque,
    /// The mesh colour is mixed by its alpha.
    Alpha,
}
