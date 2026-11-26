use crate::structs::Held;

/// One place in a resource store; the generation tells ids of its earlier users apart.
pub(crate) struct Slot<T> {
    pub(crate) generation: u32,
    pub(crate) held: Option<Held<T>>,
}
