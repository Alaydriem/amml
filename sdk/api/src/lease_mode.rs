use amml_abi::AmmlLeaseMode;

/// How a lease shares input with other leases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeaseMode {
    /// Lives beside other shared leases.
    Shared,
    /// Lives alone among the leases that hold input: refused while another lease is active, unless
    /// it outranks that lease (`Input::set_ranking`) or preempts.
    Exclusive,
}

impl From<LeaseMode> for AmmlLeaseMode {
    fn from(mode: LeaseMode) -> Self {
        match mode {
            LeaseMode::Shared => Self::SHARED,
            LeaseMode::Exclusive => Self::EXCLUSIVE,
        }
    }
}
