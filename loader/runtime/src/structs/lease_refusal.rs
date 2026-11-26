/// Why the lease book refuses a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LeaseRefusal {
    #[error("another lease holds input that this lease cannot share")]
    ExclusiveHeld,
    #[error("another preempt lease is active")]
    PreemptHeld,
    #[error("another mod owns the lease ranking")]
    RankingOwned,
}
