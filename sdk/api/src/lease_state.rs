use amml_abi::AmmlLeaseState;

/// The state of an input lease.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeaseState {
    /// The lease may inject, and the loader enforces its blocks.
    Active,
    /// Another lease preempts or outranks it; it is active again when that lease ends.
    Suspended,
    /// The lease is gone. A value this crate does not know also reads as lost.
    Lost,
}

impl From<AmmlLeaseState> for LeaseState {
    fn from(state: AmmlLeaseState) -> Self {
        match state {
            AmmlLeaseState::ACTIVE => Self::Active,
            AmmlLeaseState::SUSPENDED => Self::Suspended,
            _ => Self::Lost,
        }
    }
}
