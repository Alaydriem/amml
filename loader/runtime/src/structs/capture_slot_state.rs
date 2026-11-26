/// The state of one slot of a `CaptureRing`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaptureSlotState {
    Free,
    /// Reserved by `wants` for the present with this index; the render thread copies into it.
    Writing {
        present: u64,
    },
    /// Holds the copy of this present; `order` ranks fills, the lowest is the oldest.
    Filled {
        present: u64,
        order: u64,
    },
    /// Lent to the consumer until `give_back`.
    Taken {
        present: u64,
    },
}
