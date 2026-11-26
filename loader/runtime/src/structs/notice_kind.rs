use amml_abi::{AmmlLostReason, AmmlReleaseReason};

/// What happened to a lease, for its owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeKind {
    Suspended,
    Resumed,
    Lost(AmmlLostReason),
    Released(AmmlReleaseReason),
    /// One key (`input` is `InjectedInputReleased::KEY`) or mouse button (`BUTTON`) was released.
    ReleasedOne {
        reason: AmmlReleaseReason,
        input: u8,
        code: u16,
    },
}
