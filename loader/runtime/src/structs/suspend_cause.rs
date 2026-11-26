use crate::structs::ResourceId;

/// Why a lease is suspended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SuspendCause {
    /// A preempt lease is live; the lease resumes when it ends.
    Preempt,
    /// An exclusive lease holds that this lease cannot run beside (a higher-ranked exclusive lease, or any
    /// exclusive lease for a shared one outside the coexist pairs); the lease is judged again when that lease
    /// ends or the coexist pairs change.
    Outranked(ResourceId),
}
