use amml_abi::{AmmlJoinOutcome, AmmlLostReason, AmmlReleaseReason};

/// A notice for one mod (a lease notice, a screenshot step or a join answer), raised to the listeners of that mod only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerEvent {
    Suspended {
        lease: u64,
    },
    Resumed {
        lease: u64,
    },
    Lost {
        lease: u64,
        reason: AmmlLostReason,
    },
    Released {
        lease: u64,
        reason: AmmlReleaseReason,
    },
    /// One key (`input` is `InjectedInputReleased::KEY`) or mouse button (`BUTTON`) of the lease was released.
    ReleasedOne {
        lease: u64,
        reason: AmmlReleaseReason,
        input: u8,
        code: u16,
    },
    ScreenshotTaken {
        id: u64,
        ok: bool,
    },
    ScreenshotWritten {
        id: u64,
        ok: bool,
    },
    /// The game answered the server join `join`.
    JoinServerResult {
        join: u64,
        outcome: AmmlJoinOutcome,
    },
}
